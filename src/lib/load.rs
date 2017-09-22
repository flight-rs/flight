use wavefront::*;
use super::mesh::{MeshSource, Indexing, VertNT, Primitive};
use std::collections::HashMap;


// TODO: Result instead of default or panic
pub fn load_wavefront(obj: &Obj<SimplePolygon>) -> MeshSource<VertNT, ()> {
    let mut verts = Vec::new();
    let mut ind_look = HashMap::new();
    let mut inds = Vec::new();
    for p in obj.objects.iter().flat_map(|g| &g.groups).flat_map(|g| &g.polys) {
        let poly = p.iter().map(|i| *ind_look.entry((i.0, i.1, i.2)).or_insert_with(|| {
            verts.push(VertNT {
                pos: obj.position[i.0],
                norm: match i.2 { Some(i) => obj.normal[i], None => [0.; 3] },
                tex: match i.1 { Some(i) => obj.texture[i], None => [0.; 2] },
            });
            verts.len() as u32 - 1
        }));
        inds.extend(poly);
    }
    MeshSource {
        verts: verts,
        inds: Indexing::Inds(inds),
        prim: Primitive::TriangleList,
        mat: (),
    }
}