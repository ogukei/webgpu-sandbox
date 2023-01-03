
use std::{sync::Arc, collections::HashMap, ops::Bound};
use gltf::camera;
use nalgebra_glm as glm;

use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{JsFuture};

use crate::{fetch::fetch, console_log};

pub struct Model {
    meshes: Vec<Arc<Mesh>>,
    mesh_map: HashMap<usize, Arc<Mesh>>,
    nodes: Vec<Arc<Node>>,
    camera: Option<Camera>,
    bounding_box: Option<BoundingBox>,
}

impl Model {
    pub async fn fetch(url: &str) -> Result<Option<Arc<Self>>, JsValue> {
        let data = fetch(url).await?;
        Ok(Self::new(data))
    }

    fn new(data: Vec<u8>) -> Option<Arc<Self>> {
        // loading
        let result = gltf::import_slice(&data).ok()?;
        let (document, buffers, _images) = result;
        // construct
        let meshes = Self::make_meshes(&document, &buffers);
        let mesh_map = Self::make_mesh_map(&meshes);
        let nodes = Self::make_nodes(&document);
        let camera = Camera::new(&document);
        // post-processing
        let bounding_box = Self::calculate_bounding_box(&mesh_map, &nodes);
        let this = Self {
            meshes,
            mesh_map,
            nodes,
            camera,
            bounding_box,
        };
        Some(Arc::new(this))
    }

    fn make_meshes(document: &gltf::Document, buffers: &Vec<gltf::buffer::Data>) -> Vec<Arc<Mesh>> {
        document.meshes()
            .map(|v| Mesh::new(v, buffers))
            .collect()
    }

    fn make_nodes(document: &gltf::Document) -> Vec<Arc<Node>> {
        let Some(scene) = document.default_scene() else { return vec![] };
        Node::flatten_nodes(scene.nodes().collect())
    }

    fn make_mesh_map(meshes: &Vec<Arc<Mesh>>) -> HashMap<usize, Arc<Mesh>> {
        meshes.iter()
            .map(Arc::clone)
            .map(|v| (v.mesh_index(), v))
            .collect()
    }

    pub fn meshes(&self) -> &Vec<Arc<Mesh>> {
        &self.meshes
    }

    pub fn nodes(&self) -> &Vec<Arc<Node>> {
        &self.nodes
    }

    pub fn camera(&self) -> Option<&Camera> {
        self.camera.as_ref()
    }

    fn calculate_bounding_box(
        mesh_map: &HashMap<usize, Arc<Mesh>>,
        nodes: &Vec<Arc<Node>>
    ) -> Option<BoundingBox> {
        let mesh_map = mesh_map;
        let bounding_box: Option<BoundingBox> = nodes.iter()
            .filter_map(|v| {
                let mesh_index = v.mesh_index()?;
                let mesh = mesh_map.get(&mesh_index)?;
                let bounding_box = mesh.bounding_box();
                Some(bounding_box.transform(v.transform()))
            })
            .fold(None, |state, v| {
                let Some(state) = state else { return Some(v) };
                Some(BoundingBox::merge(&state, &v))
            });
        bounding_box
    }

    pub fn bounding_box(&self) -> Option<&BoundingBox> {
        self.bounding_box.as_ref()
    }
}

pub struct Mesh {
    mesh_index: usize,
    positions: Vec<f32>,
    indices: Vec<u32>,
    normals: Vec<f32>,
    bounding_box: BoundingBox,
}

impl Mesh {
    fn new(mesh: gltf::Mesh, buffers: &Vec<gltf::buffer::Data>) -> Arc<Self> {
        let mesh_index = mesh.index();
        // flattens primitives
        let mesh_accessors = MeshAccessor::flatten(&mesh, buffers);
        let state = FlattenMeshState::new(mesh_accessors);
        let this = Self {
            mesh_index,
            positions: state.positions,
            indices: state.indices,
            normals: state.normals,
            bounding_box: state.bounding_box.unwrap(),
        };
        Arc::new(this)
    }

    pub fn positions(&self) -> &Vec<f32> {
        &self.positions
    }

    pub fn indices(&self) -> &Vec<u32> {
        &self.indices
    }

    pub fn normals(&self) -> &Vec<f32> {
        &self.normals
    }

    pub fn mesh_index(&self) -> usize {
        self.mesh_index
    }

    pub fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }
}

pub struct Node {
    node_index: usize,
    transform: glm::Mat4,
    mesh_index: Option<usize>,
}

impl Node {
    pub fn flatten_nodes(nodes: Vec<gltf::Node>) -> Vec<Arc<Self>> {
        nodes.iter()
            .map(|v| Self::flatten(v, &glm::identity()))
            .flatten()
            .collect()
    }

    fn new(node: &gltf::Node, transform: &glm::Mat4) -> Option<Arc<Self>> {
        let node_index = node.index();
        let mesh_index = node.mesh().map(|v| v.index());
        // transform
        let matrix = node.transform().matrix();
        let local_transform: Vec<f32> = matrix.into_iter()
            .flatten()
            .collect();
        let local_transform = glm::make_mat4(&local_transform);
        let transform = transform * local_transform;
        let this = Self {
            node_index,
            mesh_index,
            transform,
        };
        Some(Arc::new(this))
    }

    fn flatten(node: &gltf::Node, transform: &glm::Mat4) -> Vec<Arc<Self>> {
        let Some(parent) = Node::new(&node, transform) else { return vec![] };
        let parent_transform = &parent.transform;
        let children: Vec<Arc<Self>> = node.children()
            .map(|v| Node::flatten(&v, parent_transform))
            .flatten()
            .collect();
        std::iter::once(parent)
            .chain(children)
            .collect()
    }

    pub fn mesh_index(&self) -> Option<usize> {
        self.mesh_index
    }

    pub fn node_index(&self) -> usize {
        self.node_index
    }

    pub fn transform(&self) -> &glm::Mat4 {
        &self.transform
    }
}

#[derive(Default)]
struct FlattenMeshState {
    pub index_offset: u32,
    pub positions: Vec<f32>,
    pub indices: Vec<u32>,
    pub normals: Vec<f32>,
    pub bounding_box: Option<BoundingBox>,
}

impl FlattenMeshState {
    fn new(meshes: Vec<MeshAccessor>) -> Self {
        let state: Self = meshes.into_iter()
            .fold(Default::default(), |mut state, v| {
                let index_offset = state.index_offset;
                let indices: Vec<u32> = v.indices().iter()
                    .map(|&v| v + index_offset)
                    .collect();
                state.positions.extend_from_slice(v.positions());
                state.indices.extend_from_slice(&indices);
                state.normals.extend_from_slice(v.normals());
                state.index_offset += (v.positions().len() / 3) as u32;
                state.bounding_box = if let Some(ref bounding_box) = state.bounding_box {
                    Some(BoundingBox::merge(v.bounding_box(), bounding_box))
                } else {
                    Some(v.bounding_box().clone())
                };
                state
            });
        state
    }
}

struct MeshAccessor {
    primitive_index: usize,
    positions: Vec<f32>,
    indices: Vec<u32>,
    normals: Vec<f32>,
    bounding_box: BoundingBox,
}

impl MeshAccessor {
    fn flatten(mesh: &gltf::Mesh, buffers: &Vec<gltf::buffer::Data>) -> Vec<Self> {
        let views: Vec<_> = mesh.primitives()
            .into_iter()
            .filter_map(|v| MeshAccessor::new(v, buffers))
            .collect();
        views
    }

    // TODO(ogukei): make this zero-copy by reading buffers directly from shader.
    fn new(primitive: gltf::Primitive, buffers: &Vec<gltf::buffer::Data>) -> Option<Self> {
        let primitive_index = primitive.index();
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        let positions: Vec<f32> = reader.read_positions()?
            .flatten()    
            .collect();
        let indices: Vec<u32> = reader.read_indices()?
            .into_u32()   
            .collect();
        let normals: Vec<f32> = reader.read_normals()?
            .flatten()
            .collect();
        let bounding_box = primitive.bounding_box();
        let bounding_box_max = glm::make_vec3(&bounding_box.max);
        let bounding_box_min = glm::make_vec3(&bounding_box.min);
        let bounding_box = BoundingBox::new(bounding_box_min, bounding_box_max);
        let this = Self {
            primitive_index,
            positions,
            indices,
            normals,
            bounding_box,
        };
        Some(this)
    }

    pub fn positions(&self) -> &Vec<f32> {
        &self.positions
    }

    pub fn indices(&self) -> &Vec<u32> {
        &self.indices
    }

    pub fn normals(&self) -> &Vec<f32> {
        &self.normals
    }

    pub fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }
}

pub struct Camera {
    position: glm::Vec3,
}

impl Camera {
    pub fn new(document: &gltf::Document) -> Option<Self> {
        let camera_node = document.nodes()
            .find_map(|v| v.camera().and(Some(v)));
        let camera_node = camera_node?;
        let (position, _, _) = camera_node.transform().decomposed();
        let position = glm::make_vec3(&position);
        let this = Self {
            position,
        };
        Some(this)
    }

    pub fn position(&self) -> &glm::Vec3 {
        &self.position
    }
}

#[derive(Clone)]
pub struct BoundingBox {
    min: glm::Vec3,
    max: glm::Vec3,
}

impl BoundingBox {
    pub fn new(min: glm::Vec3, max: glm::Vec3) -> Self {
        Self {
            min,
            max,
        }
    }

    pub fn min(&self) -> &glm::Vec3 {
        &self.min
    }

    pub fn max(&self) -> &glm::Vec3 {
        &self.max
    }

    pub fn merge(lhs: &Self, rhs: &Self) -> Self {
        let min = glm::min2(&lhs.min, &rhs.min);
        let max = glm::max2(&lhs.max, &rhs.max);
        Self::new(min, max)
    }

    pub fn transform(&self, matrix: &glm::Mat4) -> Self {
        let min = &self.min;
        let max = &self.max;
        let min = glm::vec4(min.x, min.y, min.z, 1.0);
        let max = glm::vec4(max.x, max.y, max.z, 1.0);
        let min = matrix * min;
        let max = matrix * max;
        Self::new(min.xyz(), max.xyz())
    }
}
