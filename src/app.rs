use crate::data::{DATA_PATH, load_items, save_items};
use crate::graph::{AppGraph, DEFAULT_IMAGE, load_texture};
use crate::node::NodeData;
use egui::CentralPanel;
use petgraph::prelude::NodeIndex;
use std::path::Path;

const ZOOM_SPEED: f32 = 0.02;

// what the user types in the "Add node" window
#[derive(Default)]
struct NewNodeForm {
    parent_name: String,
    name: String,
    price: f32,
    image: String,
}

pub struct App {
    graph: AppGraph,
    // Some = the "Node properties" window is open for a spafic node index
    editing: Option<NodeIndex>,
    new_node: Option<NewNodeForm>, // Some = the "Add node" window is open
    relayout: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            graph: AppGraph::build_graph(&cc.egui_ctx, &load_items(DATA_PATH)),
            editing: None,
            new_node: None,
            relayout: false,
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.spacing_mut().button_padding = egui::vec2(10.0, 4.0);

            if ui
                .button("💾 Save")
                .on_hover_text(format!("Save to {DATA_PATH}"))
                .clicked()
            {
                save_items(DATA_PATH, &self.graph.to_items());
            }

            ui.separator();

            if ui
                .button("➕ Add Node")
                .on_hover_text("Add a new node")
                .clicked()
            {
                self.new_node = Some(NewNodeForm::default());
            }
        });
        ui.separator();
    }

    fn graph_view(&mut self, ui: &mut egui::Ui) {
        if self.relayout {
            egui_graphs::reset_layout::<egui_graphs::LayoutStateHierarchical>(ui, None);
            self.relayout = false;
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
        .show(ui, &mut self.graph.inner);

        if let Some(&idx) = self.graph.inner.selected_nodes().first() {
            self.editing = Some(idx);
        }
    }

    fn properties_window(&mut self, ctx: &egui::Context) {
        let Some(idx) = self.editing else {
            return;
        };

        let mut close = false;
        let mut new_label = None;

        egui::Window::new("Node properties").show(ctx, |ui| {
            if let Some(node) = self.graph.inner.node_mut(idx) {
                let mut label = node.label();
                if ui.text_edit_singleline(&mut label).changed() {
                    new_label = Some(label);
                }

                ui.horizontal(|ui| {
                    ui.label("Price:");
                    ui.add(
                        egui::DragValue::new(&mut node.payload_mut().price)
                            .speed(0.0)
                            .max_decimals(2),
                    );
                });
                ui.horizontal(|ui| close = ui.button("close").clicked());

                if close {
                    node.set_selected(false);
                    self.editing = None;
                }
            }
        });

        if let Some(label) = new_label {
            self.graph.change_node_name(idx, &label);
        }
    }

    fn new_node_window(&mut self, ctx: &egui::Context) {
        let Some(form) = &mut self.new_node else {
            return;
        };

        let name = form.name.trim();
        let name_taken = self
            .graph
            .inner
            .nodes_iter()
            .any(|(_, n)| n.label() == name);

        let mut create = false;
        let mut cancel = false;

        egui::Window::new("Add node")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                egui::Grid::new("new_node_form")
                    .num_columns(2)
                    .spacing([8.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut form.name);
                        ui.end_row();

                        ui.label("Price:");
                        ui.add(
                            egui::DragValue::new(&mut form.price)
                                .speed(0.0)
                                .max_decimals(2),
                        );
                        ui.end_row();

                        ui.label("Image:");
                        ui.add(
                            egui::TextEdit::singleline(&mut form.image)
                                .hint_text("assets/... (optional)"),
                        );
                        ui.end_row();

                        ui.label("parent name:");
                        ui.add(
                            egui::TextEdit::singleline(&mut form.parent_name)
                                .hint_text("parent name"),
                        );
                    });

                if name_taken {
                    ui.colored_label(
                        ui.visuals().error_fg_color,
                        "A node with this name already exists",
                    );
                }

                ui.separator();
                ui.horizontal(|ui| {
                    let valid = !form.name.trim().is_empty() && !name_taken;
                    create = ui.add_enabled(valid, egui::Button::new("Create")).clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
        if cancel {
            self.new_node = None;
        } else if create {
            let form = self.new_node.take().unwrap();
            let image = Some(form.image.trim().to_string()).filter(|path| !path.is_empty());
            let texture_path = image
                .as_deref()
                .filter(|path| Path::new(path).is_file())
                .unwrap_or(DEFAULT_IMAGE);

            let data = NodeData {
                texture: load_texture(ctx, texture_path),
                image,
                price: form.price,
                children: Vec::new(),
            };
            self.graph
                .add_node(data, form.name.trim().to_string(), &form.parent_name);
            self.relayout = true;
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        CentralPanel::default().show(ui, |ui| {
            self.toolbar(ui);
            self.graph_view(ui);
            self.properties_window(ui.ctx());
            self.new_node_window(ui.ctx());
        });
    }
}
