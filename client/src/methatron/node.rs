use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock, Weak};

use crate::methatron::{
  material::{self, Material, MaterialUserData},
  drawable::Drawable,
  math::matrix
};

static NODE_ID: AtomicU64 = AtomicU64::new(0);

pub fn new() -> Node {
  let transform = matrix::new();

  let node = Arc::new(RwLock::new(ImplNode {
    _id: NODE_ID.fetch_add(1, Ordering::SeqCst),
    me: Weak::new(),
    is_disposed: false,
    network_id: String::new(),
    parent: None,
    transform: transform,
    world_transform: matrix::new(),
    drawable: None,
    children: HashMap::new(),
    material: material::new(),
  }));

  node.write().unwrap().me = Arc::downgrade(&node);

  node
}

pub struct ImplNode {
  _id: u64,
  me: Weak<RwLock<ImplNode>>,
  pub is_disposed: bool,
  pub network_id: String,
  pub parent: Option<Node>,
  pub transform: matrix::Matrix,
  pub world_transform: matrix::Matrix,
  pub drawable: Option<Drawable>,
  pub material: Material,
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

  pub fn remove_child(&mut self, child: Node) {
    let id = {
      let child = child.read().unwrap();
      child._id
    };
    if let Some(_) = self.children.remove(&id) {
      let mut child = child.write().unwrap();
      child.parent = None;
    }
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
    {
      let mut wt = self.world_transform.lock().unwrap();
      matrix::identity(&mut wt);
      matrix::mul_assign(&mut wt, &root.lock().unwrap());
      let t = self.transform.lock().unwrap();
      matrix::mul_assign(&mut wt, &t);
    }

    for c in &self.children {
      c.1.read().unwrap().update_world_transform(&self.world_transform);
    }
  }

  pub fn dispose(&mut self) {
    if !self.is_disposed {
      if let Some(drawable) = &self.drawable {
        let mut d = drawable.write().unwrap();
        d.references.remove(&self._id);
      }
      self.drawable = None;

      if let Some(ref parent) = self.parent {
        let mut parent = parent.write().unwrap();
        parent.children.remove(&self._id);
      }
      self.parent = None;
      self.is_disposed = true;
    }
  }
}

impl Drop for ImplNode {
  fn drop(&mut self) {
    self.dispose();
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
      node.set_drawable(bd.0.clone());

      Ok(())
    });

    methods.add_method("get_material", |_, this, ()| {
      let node = this.node.read().unwrap();
      Ok(MaterialUserData(node.material.clone()))
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

    methods.add_method("remove_child", |_, this, child: mlua::AnyUserData| {
      let mut node = this.node.write().unwrap();
      let child = child.borrow::<NodeUserData>().unwrap();

      node.remove_child(child.node.clone());

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
