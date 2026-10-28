use egui::{CentralPanel, Color32, Pos2, Rect, Shape, TextureHandle, Vec2};
use egui_graphs::{DisplayNode, DrawContext, Graph, NodeProps};
use petgraph::prelude::NodeIndex;
use petgraph::stable_graph::{DefaultIx, IndexType};
use petgraph::{EdgeType, Undirected};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const DEFAULT_IMAGE: &str = "src/default.png";
const NODE_SIZE: f32 = 25.0;
const ZOOM_SPEED: f32 = 0.02;

//what inside the json file looks like
#[derive(Debug, Deserialize, Serialize)]
struct Item {
    name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    children: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    #[serde(default)]
    price: f32,
}

//what we store in the graph node
#[derive(Clone)]
struct NodeData {
    texture: TextureHandle,
    image: Option<String>, // the path from the JSON, not the fallback
    price: f32,
    children: Vec<NodeIndex>,
}

#[derive(Clone)]
struct ImageNode {
    location: Pos2,
    texture: TextureHandle,
}

impl From<NodeProps<NodeData>> for ImageNode {
    fn from(props: NodeProps<NodeData>) -> Self {
        Self {
            location: props.location(),
            texture: props.payload.texture.clone(), // was props.payload.clone()
        }
    }
}

impl<E: Clone, Ty: EdgeType, Ix: IndexType> DisplayNode<NodeData, E, Ty, Ix> for ImageNode {
    fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
        self.location + dir.normalized() * (NODE_SIZE / 2.0)
    }

    fn is_inside(&self, pos: Pos2) -> bool {
        (pos - self.location).length() <= NODE_SIZE / 2.0
    }

    fn update(&mut self, state: &NodeProps<NodeData>) {
        self.location = state.location();
    }

    fn shapes(&mut self, ctx: &DrawContext) -> Vec<Shape> {
        let center = ctx.meta.canvas_to_screen_pos(self.location);
        let side = ctx.meta.canvas_to_screen_size(NODE_SIZE);
        let rect = Rect::from_center_size(center, Vec2::splat(side));
        let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
        vec![Shape::image(self.texture.id(), rect, uv, Color32::WHITE)]
    }
}

type G = Graph<NodeData, (), Undirected, DefaultIx, ImageNode>;

struct App {
    graph: G,
    editing: Option<NodeIndex>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            graph: generate_graph(&cc.egui_ctx),
            editing: None,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        CentralPanel::default().show(ui, |ui| {
            if ui.button("Save").clicked() {
                save_graph(&self.graph, "src/data.json");
            }

            egui_graphs::GraphView::<
                egui_graphs::LayoutStateHierarchical,
                egui_graphs::LayoutHierarchical,
            >::new()
            .with_navigations(
                &egui_graphs::SettingsNavigation::default()
                    .with_zoom_and_pan_enabled(true)
                    .with_fit_to_screen_enabled(false)
                    .with_zoom_speed(ZOOM_SPEED),
            )
            .with_interactions(
                &egui_graphs::SettingsInteraction::default()
                    .with_dragging_enabled(false)
                    .with_node_selection_enabled(true),
            )
            .show(ui, &mut self.graph);

            if let Some(&idx) = self.graph.selected_nodes().first() {
                self.editing = Some(idx);
            }

            if let Some(idx) = self.editing {
                egui::Window::new("Node properties").show(ui.ctx(), |ui| {
                    if let Some(node) = self.graph.node_mut(idx) {
                        let mut label = node.label();
                        if ui.text_edit_singleline(&mut label).changed() {
                            node.set_label(label);
                        }

                        ui.horizontal(|ui| {
                            ui.label("Price:");
                            ui.add(
                                egui::DragValue::new(&mut node.payload_mut().price)
                                    .speed(0.0)
                                    .max_decimals(2),
                            );
                        });
                    }
                });
            }
        });
    }
}

fn load_texture_from_path(ctx: &egui::Context, path: &str) -> TextureHandle {
    let image = image::open(path)
        .unwrap_or_else(|e| panic!("can't open {path}: {e}"))
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
    ctx.load_texture(path, color, egui::TextureOptions::LINEAR)
}

fn save_graph(graph: &G, path: &str) {
    println!("Saving graph to {path}");
    let items: Vec<Item> = graph
        .nodes_iter()
        .map(|(_, node)| {
            let data = node.payload();
            Item {
                name: node.label(),
                children: data
                    .children
                    .iter()
                    .map(|&c| graph.node(c).unwrap().label())
                    .collect(),
                image: data.image.clone(),
                price: data.price,
            }
        })
        .collect();

    let json = serde_json::to_string_pretty(&items).unwrap();
    fs::write(path, json).unwrap();
}

fn generate_graph(ctx: &egui::Context) -> G {
    let mut graph: G = Graph::new();

    let data = fs::read_to_string("src/data.json").unwrap();
    let items = from_str::<Vec<Item>>(&data).unwrap();

    let mut textures: HashMap<String, TextureHandle> = HashMap::new();
    let mut ids: HashMap<String, NodeIndex> = HashMap::with_capacity(items.len());

    let default_texture = load_texture_from_path(ctx, DEFAULT_IMAGE);

    for item in &items {
        let texture = match item
            .image
            .as_deref()
            .filter(|path| Path::new(path).is_file())
        {
            Some(path) => textures
                .entry(path.to_string())
                .or_insert_with(|| load_texture_from_path(ctx, path))
                .clone(),
            None => default_texture.clone(),
        };

        let data = NodeData {
            texture,
            image: item.image.clone(),
            price: item.price,
            children: Vec::new(),
        };
        let id = graph.add_node_with_label(data, item.name.clone());

        ids.insert(item.name.clone(), id);
    }

    for item in &items {
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

    graph
}

fn main() {
    eframe::run_native(
        "egui_graphs_basic_demo",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
    .unwrap();
}
