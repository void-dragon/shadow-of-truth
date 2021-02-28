use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock, Weak};

use crate::methatron::{
  drawable::Drawable,
  math::matrix
};

static NODE_ID: AtomicU64 = AtomicU64::new(0);

pub fn new() -> Node {
  let transform = matrix::new();

  let node = Arc::new(RwLock::new(ImplNode {
    _id: NODE_ID.fetch_add(1, Ordering::SeqCst),
    me: Weak::new(),
    network_id: String::new(),
    parent: None,
    transform: transform,
    world_transform: matrix::new(),
    drawable: None,
    children: HashMap::new(),
  }));

  node.write().unwrap().me = Arc::downgrade(&node);

  node
}

pub struct ImplNode {
  _id: u64,
  me: Weak<RwLock<ImplNode>>,
  pub network_id: String,
  pub parent: Option<Node>,
  pub transform: matrix::Matrix,
  pub world_transform: matrix::Matrix,
  pub drawable: Option<Drawable>,
  pub children: HashMap<u64, Node>,
}

impl ImplNode {
  pub fn id(&self) -> u64 {
    self._id
  }

  pub fn add_child(&mut self, child: Node) {
    let id = {
      let mut child = child.write().unwrap();

      if let Some(parent) = &child.parent {
        parent.write().unwrap().children.remove(&child._id);
      }
      child.parent = Some(self.me.upgrade().unwrap());

      child._id
    };
    self.children.insert(id, child);
  }

  pub fn set_drawable(&mut self, drawable: Drawable) {
    if let Some(drawable) = &self.drawable {
      let mut d = drawable.write().unwrap();
      d.references.remove(&self._id);
    }

    {
      let mut d = drawable.write().unwrap();
      d.references.insert(self._id, self.me.upgrade().unwrap());
    }
    self.drawable = Some(drawable);
  }

  pub fn update_world_transform(&self, root: &matrix::Matrix) {
    matrix::identity(&self.world_transform);
    matrix::mul_assign(&self.world_transform, root);
    matrix::mul_assign(&self.world_transform, &self.transform);

    for c in &self.children {
      c.1.read().unwrap().update_world_transform(&self.world_transform);
    }
  }
}

pub type Node = Arc<RwLock<ImplNode>>;

pub struct NodeUserData {
  pub node: Node,
}

impl mlua::UserData for NodeUserData {
  fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("id", |_, this, ()| {
      Ok(this.node.read().unwrap().id())
    });

    methods.add_method("network_id", |_, this, ()| {
      Ok(this.node.read().unwrap().network_id.clone())
    });

    methods.add_method("set_drawable", |_, this, drawable: mlua::AnyUserData| {
      use crate::methatron::drawable::DrawableUserData;

      let mut node = this.node.write().unwrap();
      let bd = drawable.borrow::<DrawableUserData>().unwrap();
      node.set_drawable(bd.drawable.clone());

      Ok(())
    });

    methods.add_method("get_parent", |_, this, _: ()| {
      let node = this.node.read().unwrap();
      if let Some(parent) = &node.parent {
        Ok(Some(NodeUserData { node: parent.clone() }))
      }
      else {
        Ok(None)
      }
    });

    methods.add_method("add_child", |_, this, child: mlua::AnyUserData| {
      let mut node = this.node.write().unwrap();
      let child = child.borrow::<NodeUserData>().unwrap();

      node.add_child(child.node.clone());

      Ok(())
    });

    methods.add_method("get_transform", |_, this, _: ()| {
      use crate::methatron::math::matrix::MatrixUserData;

      let node = this.node.read().unwrap();

      Ok(MatrixUserData { matrix: node.transform.clone() })
    });
  }
}

pub fn load_module(lua: &mlua::Lua, ns: &mlua::Table) -> mlua::Result<()> {
  let module = lua.create_table()?;

  let new_node = lua.create_function(|_,_: ()| {
    Ok(NodeUserData { node: new() })
  })?;
  module.set("new", new_node)?;

  ns.set("node", module)?;

  Ok(())
}
