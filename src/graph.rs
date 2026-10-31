use crate::data::Item;
use crate::node::{ImageNode, NodeData};
use crate::short_numbers::{conver_to_short_number, convert_to_regular_number};
use egui::TextureHandle;
use egui_graphs::Graph;
use petgraph::Undirected;
use petgraph::prelude::NodeIndex;
use petgraph::stable_graph::DefaultIx;
use std::collections::HashMap;
use std::path::Path;

pub const DEFAULT_IMAGE: &str = "assets/default.png";

pub type InnerGraph = Graph<NodeData, (), Undirected, DefaultIx, ImageNode>;

pub struct AppGraph {
    pub inner: InnerGraph,
    ids: HashMap<String, NodeIndex>,
}

impl AppGraph {
    pub fn build_graph(ctx: &egui::Context, items: &[Item]) -> Self {
        let mut graph: InnerGraph = Graph::new();

        let mut textures: HashMap<String, TextureHandle> = HashMap::new();
        let mut ids: HashMap<String, NodeIndex> = HashMap::with_capacity(items.len());

        let default_texture = load_texture(ctx, DEFAULT_IMAGE);

        for item in items {
            let texture = match item
                .image
                .as_deref()
                .filter(|path| Path::new(path).is_file())
            {
                // if the image path is valid, load the texture and cache it
                Some(path) => textures
                    .entry(path.to_string())
                    .or_insert_with(|| load_texture(ctx, path))
                    .clone(),
                None => default_texture.clone(),
            };

            let data = NodeData {
                texture,
                image: item.image.clone(),
                price: convert_to_regular_number(&item.price)
                    .unwrap_or_else(|| panic!("bad price {:?} for {}", item.price, item.name))
                    as f32,
                children: Vec::new(),
                machine: item.machine.clone(),
            };
            let id = graph.add_node_with_label(data, item.name.clone());

            ids.insert(item.name.clone(), id);
        }

        for item in items {
            // add edges and store children as ids, so renaming a node won't break the links
            let parent = ids[&item.name];
            for child_name in &item.children {
                let child = ids[child_name];
                graph.add_edge(parent, child, ());
                graph
                    .node_mut(parent)
                    .unwrap()
                    .payload_mut()
                    .children
                    .push(child);
            }
        }

        Self { inner: graph, ids }
    }

    pub fn change_node_name(&mut self, id: NodeIndex, new_name: &str) {
        let node = self.inner.node_mut(id).unwrap();
        let old_name = node.label();
        node.set_label(new_name.to_string());
        self.ids.remove(&old_name);
        self.ids.insert(new_name.to_string(), id);
    }

    // the reverse of build_graph for saving the graph to a json file
    pub fn to_items(&self) -> Vec<Item> {
        self.inner
            .nodes_iter()
            .map(|(_, node)| {
                let data = node.payload();
                Item {
                    name: node.label(),
                    children: data
                        .children
                        .iter()
                        .map(|&c| self.inner.node(c).unwrap().label())
                        .collect(),
                    image: data.image.clone(),
                    price: conver_to_short_number(data.price as f64),
                    machine: data.machine.clone(),
                }
            })
            .collect()
    }

    pub fn add_node(&mut self, node_data: NodeData, name: String, parent_name: &str) {
        let parent_id = self.ids[parent_name];

        let id = self.inner.add_node_with_label(node_data, name.clone());
        self.ids.insert(name, id);

        self.inner.add_edge(parent_id, id, ());
        self.inner
            .node_mut(parent_id)
            .unwrap()
            .payload_mut()
            .children
            .push(id);
    }

    pub fn multiply_prices(&mut self, scalar: f32) {
        for node in self.inner.g_mut().node_weights_mut() {
            node.payload_mut().price *= scalar;
        }
    }
}

pub fn load_texture(ctx: &egui::Context, path: &str) -> TextureHandle {
    let image = image::open(path)
        .unwrap_or_else(|e| panic!("can't open {path}: {e}"))
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
    ctx.load_texture(path, color, egui::TextureOptions::LINEAR)
}
