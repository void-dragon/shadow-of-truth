const xml = require('xml-js')
const fs = require('fs')
const crypto = require('crypto')

const INPUT = process.argv[2];
const OUTPUT = process.argv[3];

console.log('INPUT:', INPUT);
console.log('OUTPUT:', OUTPUT);

let data = xml.xml2json(fs.readFileSync(INPUT), {compact: true})
data = JSON.parse(data)

geometry = data.COLLADA.library_geometries.geometry
name = geometry._attributes.id
verticeSource = geometry.mesh.source.find(s => s._attributes.id === name + '-positions')
normalSource = geometry.mesh.source.find(s => s._attributes.id === name + '-normals')
faces = geometry.mesh.triangles.p._text.split(' ').map(x => parseInt(x))

vertices = verticeSource.float_array._text.split(' ').map(x => parseFloat(x))
normals = normalSource.float_array._text.split(' ').map(x => parseFloat(x))

faceCount =  geometry.mesh.triangles._attributes.count
vertexBuffer = []
normalBuffer = []
indexBuffer = []
tupleMap = {}

function add2(dst, src, idx) {
    for(let i = idx * 3, e = idx * 3 + 3; i < e; i++) {
        dst.push(src[i])
    }
}

for(let i = 0; i < faces.length; i+=2) {
    const tuple = []
    const hash = crypto.createHash('sha256')
    const vidx = faces[i]
    const nidx = faces[i+1]

    add2(tuple, vertices, vidx)
    add2(tuple, normals, nidx)

    hash.update(new Float32Array(tuple))
    const digest = hash.digest('hex')

    console.log(i, faces[i], faces[i+1])

    let n = tupleMap[digest]

    if (!n) {
        n = vertexBuffer.length / 3
        add2(vertexBuffer, vertices, vidx)
        add2(normalBuffer, normals, nidx)
        tupleMap[digest] = n
    }

    indexBuffer.push(n)
}

console.log('vertices', vertexBuffer.length)
console.log('normals', normalBuffer.length)
console.log('indices', indexBuffer.length)
evermoreObj = {
    name,
    indices: indexBuffer,
    vertices: vertexBuffer,
    normals: normalBuffer
}

fs.writeFileSync(OUTPUT, JSON.stringify(evermoreObj))
//console.log(vertices)
// console.log(normals)
