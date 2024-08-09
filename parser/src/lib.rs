use serde::Serialize;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[derive(Serialize)]
pub struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Serialize)]
pub struct Model {
    vertices: Vec<Vertex>,
    faces: Vec<u32>,
}

#[derive(Serialize)]
pub struct Surface {
    id: String,
    name: Option<String>,
    surface_type: String,
    model: Model,
    openings: Vec<Opening>,
}

// name can be undefined
#[derive(Serialize)]
pub struct Opening {
    id: String,
    name: Option<String>,
    surface_type: String,
    model: Model,
}

#[derive(Serialize)]
pub struct Models {
    surfaces: Vec<Surface>,
}

#[derive(Serialize)]
pub struct Face {
    vertices: Vec<Vec<f32>>,
    index: Vec<u32>,
}

#[derive(Serialize)]
pub struct ShellSurface {
    surface_type: String,
    faces: Vec<Face>,
}

#[derive(Serialize)]
pub struct AnalyticalShell {
    shell_surfaces: Vec<ShellSurface>,
}
#[derive(Serialize)]
pub struct ClosedShell {
    faces: Vec<Face>,
}

#[derive(Serialize)]
pub struct ShellGeometry {
    id: String,
    unit: String,
    closed_shell: Option<ClosedShell>,
    analytical_shell: Option<AnalyticalShell>,
}

#[derive(Serialize)]
pub struct Space {
    id: String,
    zone_id_ref: String,
    name: String,
    area: f32,
    volume: f32,
    shell_geometry: ShellGeometry,
    cad_object_id: i32,
}
#[derive(Serialize)]
pub struct Building {
    id: String,
    building_type: String,
    area: f32,
    spaces: Vec<Space>,
}

pub fn attribute_from_node(node: roxmltree::Node, attribute_name: &str) -> String {
    node.children()
        .find(|n| n.has_tag_name(attribute_name))
        .unwrap()
        .text()
        .unwrap()
        .to_string()
}

pub fn faces_from_polyloop(node: roxmltree::Node) -> Vec<Face> {
    for child in node.children() {
        if child.has_tag_name("PolyLoop") {
            return faces_from_polyloop(child);
        }
    }
    let mut faces = Vec::new();
    let mut temp_vertices = Vec::new();
    for child in node.children() {
        if child.has_tag_name("CartesianPoint") {
            let mut coords = child.children();
            let x = coords
                .next()
                .unwrap()
                .text()
                .unwrap()
                .parse::<f32>()
                .unwrap();
            let z = coords
                .next()
                .unwrap()
                .text()
                .unwrap()
                .parse::<f32>()
                .unwrap();
            let y = coords
                .next()
                .unwrap()
                .text()
                .unwrap()
                .parse::<f32>()
                .unwrap();
            temp_vertices.push(vec![x, y, z]);
        }
    }
    let mut vertex_indices = Vec::new();
    let mut index = Vec::new();
    let mut vertices: Vec<Vec<f32>> = Vec::new();
    for vertex in temp_vertices {
        let mut index = 0;
        let mut found = false;
        for (i, v) in vertices.iter().enumerate() {
            if v[0] == vertex[0] && v[1] == vertex[1] && v[2] == vertex[2] {
                index = i as u32;
                found = true;
                break;
            }
        }
        if !found {
            vertices.push(vertex);
            index = (vertices.len() - 1) as u32;
        }
        vertex_indices.push(index);
    }
    for i in 2..vertex_indices.len() {
        index.push(vertex_indices[0]);
        index.push(vertex_indices[i - 1]);
        index.push(vertex_indices[i]);
    }
    faces.push(Face { vertices, index });
    faces
}

pub fn shell_geometry_from_node(node: roxmltree::Node) -> ShellGeometry {
    for child in node.children() {
        if child.has_tag_name("ShellGeometry") {
            return shell_geometry_from_node(child);
        }
    }
    let id = node.attribute("id").unwrap().to_string();
    let unit = node.attribute("unit").unwrap().to_string();
    let mut closed_shell = None;
    let mut analytical_shell = None;

    for child in node.children() {
        if child.has_tag_name("ClosedShell") {
            closed_shell = Some(ClosedShell {
                faces: faces_from_polyloop(child),
            });
        } else if child.has_tag_name("AnalyticalShell") {
            let mut shell_surfaces = Vec::new();
            for analytical_shell_child in child.children() {
                if analytical_shell_child.has_tag_name("ShellSurface") {
                    let surface_type = analytical_shell_child
                        .attribute("surfaceType")
                        .unwrap()
                        .to_string();
                    let faces = faces_from_polyloop(analytical_shell_child);
                    shell_surfaces.push(ShellSurface {
                        surface_type,
                        faces,
                    });
                }
            }
            analytical_shell = Some(AnalyticalShell { shell_surfaces });
        }
    }
    ShellGeometry {
        id,
        unit,
        closed_shell,
        analytical_shell,
    }
}

pub fn spaces_from_node(node: roxmltree::Node) -> Vec<Space> {
    let mut spaces = Vec::new();
    for child in node.children() {
        if child.has_tag_name("Space") {
            let id = child.attribute("id").unwrap().to_string();
            let zone_id_ref = child.attribute("zoneIdRef").unwrap().to_string();
            let name = attribute_from_node(child, "Name");
            let area = attribute_from_node(child, "Area").parse::<f32>().unwrap();
            let volume = attribute_from_node(child, "Volume").parse::<f32>().unwrap();
            let shell_geometry = shell_geometry_from_node(child);
            let cad_object_id = attribute_from_node(child, "CADObjectId")
                .parse::<i32>()
                .unwrap();
            spaces.push(Space {
                id,
                zone_id_ref,
                name,
                area,
                volume,
                shell_geometry,
                cad_object_id,
            });
        }
    }
    spaces
}

// #[wasm_bindgen]
// pub fn parse_bem(bem: &str) -> Result<JsValue, JsValue> {
//     let result = roxmltree::Document::parse(bem);
//     match result {
//         Ok(xml) => {
//             let mut buildings = Vec::new();
//             for node in xml.descendants() {
//                 if node.has_tag_name("Building") {
//                     let building = Building {
//                         id: node.attribute("id").unwrap().to_string(),
//                         building_type: node.attribute("buildingType").unwrap().to_string(),
//                         area: attribute_from_node(node, "Area").parse::<f32>().unwrap(),
//                         spaces: spaces_from_node(node),
//                     };
//                     buildings.push(building);
//                 }
//             }
//             match to_value(&buildings) {
//                 Ok(js_value) => Ok(js_value),
//                 Err(err) => {
//                     // Handle the error during serialization
//                     let error_message = format!("Failed to serialize BEM: {}", err);
//                     Err(JsValue::from_str(&error_message))
//                 }
//             }
//         }
//         Err(err) => {
//             let error_message = format!("Failed to parse BEM: {}", err);
//             Err(JsValue::from_str(&error_message))
//         }
//     }
// }

#[wasm_bindgen]
pub fn parse_bem(bem: &str) -> Result<JsValue, JsValue> {
    let result = roxmltree::Document::parse(bem);
    match result {
        Ok(xml) => {
            let mut surfaces = Vec::new();
            for node in xml.descendants() {
                if node.has_tag_name("Surface") {
                    let mut openings = Vec::new();
                    let id = node.attribute("id").unwrap().to_string();
                    let mut name = None;
                    let surface_type = if node.has_tag_name("Surface") {
                        node.attribute("surfaceType").unwrap().to_string()
                    } else {
                        node.attribute("openingType").unwrap().to_string()
                    };

                    let mut vertices = Vec::new();
                    let mut faces = Vec::new();
                    for child in node.children() {
                        if child.has_tag_name("Name") {
                            name = Some(child.text().unwrap().to_string());
                        }
                        if child.has_tag_name("PlanarGeometry") {
                            for planar_geometry in child.children() {
                                if planar_geometry.has_tag_name("PolyLoop") {
                                    polyloop_to_model(planar_geometry, &mut vertices, &mut faces);
                                }
                            }
                        }
                        if child.has_tag_name("Opening") {
                            let id = child.attribute("id").unwrap().to_string();
                            let mut name = None;
                            let surface_type = child.attribute("openingType").unwrap().to_string();
                            let mut vertices = Vec::new();
                            let mut faces = Vec::new();
                            for opening_child in child.children() {
                                if opening_child.has_tag_name("Name") {
                                    name = Some(opening_child.text().unwrap().to_string());
                                }
                                if opening_child.has_tag_name("PlanarGeometry") {
                                    for planar_geometry in opening_child.children() {
                                        if planar_geometry.has_tag_name("PolyLoop") {
                                            polyloop_to_model(
                                                planar_geometry,
                                                &mut vertices,
                                                &mut faces,
                                            );
                                        }
                                    }
                                }
                            }
                            let model = Model { vertices, faces };
                            openings.push(Opening {
                                model,
                                name,
                                surface_type,
                                id,
                            });
                        }
                    }
                    let model = Model { vertices, faces };
                    if node.has_tag_name("Surface") {
                        surfaces.push(Surface {
                            model,
                            name,
                            surface_type,
                            id,
                            openings,
                        });
                    }
                }
            }
            let models = Models { surfaces };
            match to_value(&models) {
                Ok(js_value) => Ok(js_value),
                Err(err) => {
                    // Handle the error during serialization
                    let error_message = format!("Failed to serialize BEM: {}", err);
                    Err(JsValue::from_str(&error_message))
                }
            }
        }
        Err(err) => {
            let error_message = format!("Failed to parse BEM: {}", err);
            Err(JsValue::from_str(&error_message))
        }
    }
}

pub fn polyloop_to_model(child: roxmltree::Node, vertices: &mut Vec<Vertex>, faces: &mut Vec<u32>) {
    let mut temp_vertices = Vec::new();
    for vertex in child.children() {
        if vertex.has_tag_name("CartesianPoint") {
            let mut coords = vertex.children();
            let x = coords
                .next()
                .unwrap()
                .text()
                .unwrap()
                .parse::<f32>()
                .unwrap();
            let z = coords
                .next()
                .unwrap()
                .text()
                .unwrap()
                .parse::<f32>()
                .unwrap();
            let y = coords
                .next()
                .unwrap()
                .text()
                .unwrap()
                .parse::<f32>()
                .unwrap();
            temp_vertices.push(Vertex { x, y, z });
        }
    }
    let mut vertex_indices = Vec::new();
    for vertex in temp_vertices {
        let mut index = 0;
        let mut found = false;
        for (i, v) in vertices.iter().enumerate() {
            if v.x == vertex.x && v.y == vertex.y && v.z == vertex.z {
                index = i as u32;
                found = true;
                break;
            }
        }
        if !found {
            vertices.push(vertex);
            index = (vertices.len() - 1) as u32;
        }
        vertex_indices.push(index);
    }
    for i in 2..vertex_indices.len() {
        faces.push(vertex_indices[0]);
        faces.push(vertex_indices[i - 1]);
        faces.push(vertex_indices[i]);
    }
}
