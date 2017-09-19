use wavefront::*;
use object::{ObjectSource, Indexing};
use defines::VertN;
use gfx::Primitive;
use std::collections::HashMap;


// TODO: Result instead of default or panic
pub fn load_wavefront(obj: &Obj<SimplePolygon>) -> ObjectSource<VertN> {
    let mut verts = Vec::new();
    let mut ind_look = HashMap::new();
    let mut inds = Vec::new();
    for p in obj.objects.iter().flat_map(|g| &g.groups).flat_map(|g| &g.polys) {
        let poly = p.iter().map(|i| *ind_look.entry((i.0, i.1, i.2)).or_insert_with(|| {
            verts.push(VertN {
                pos: obj.position[i.0],
                norm: match i.2 { Some(i) => obj.normal[i], None => [0.; 3] },
            });
            verts.len() as u32 - 1
        }));
        inds.extend(poly);
    }
    ObjectSource {
        verts: verts,
        inds: Indexing::Inds(inds),
        prim: Primitive::TriangleList,
    }
}