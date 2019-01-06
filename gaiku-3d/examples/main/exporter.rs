extern crate obj_exporter;

use obj_exporter::{
    Geometry,
    ObjSet,
    Object,
    Primitive,
    Shape,
    Vertex,
};

use gaiku_3d::common::{
    Mesh,
    Vec3,
};

pub fn to_obj(mesh: &Mesh, position: &Vec3<i32>, name: &str) -> Object {
    let mut vertices = vec![];
    let mut indices = vec![];

    for vertex in mesh.vertices.iter() {
        let x = vertex.x as f64 + position.x as f64;
        let y = vertex.y as f64 + position.y as f64;
        let z = vertex.z as f64 + position.z as f64;
        vertices.push((x, y, z));
    }

    for i in (0..mesh.indices.len()).step_by(3) {
        indices.push((mesh.indices[i], mesh.indices[i + 1], mesh.indices[i + 2]))
    }

    Object {
        name: name.to_owned(),
        vertices: vertices.into_iter()
            .map(|(x, y, z)| Vertex { x, y, z })
            .collect(),
        tex_vertices: vec![],
        normals: vec![],
        geometry: vec![
            Geometry {
                material_name: None,
                shapes: indices
                    .into_iter()
                    .map(|(x, y, z)| Shape {
                        primitive: Primitive::Triangle((x, None, None), (y, None, None), (z, None, None)),
                        groups: vec![],
                        smoothing_groups: vec![],
                    })
                    .collect(),
            },
        ],
    }
}

pub fn export(data: Vec<(Mesh, Vec3<i32>)>, name: &str) {
    let mut objects = vec![];

    for (index, (mesh,  position)) in data.iter().enumerate() {
        objects.push(to_obj(mesh, position,&format!("mesh_{}", index)));
    }

    let set = ObjSet {
        material_library: None,
        objects
    };

    obj_exporter::export_to_file(&set, format!("examples/output/{}.obj", name)).unwrap();
}