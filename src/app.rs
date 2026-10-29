use crate::data::{DATA_PATH, load_items, save_items};
use crate::graph::{AppGraph, DEFAULT_IMAGE, load_texture};
use crate::node::NodeData;
use crate::short_numbers::{conver_to_short_number, convert_to_regular_number};
use egui::CentralPanel;
use petgraph::prelude::NodeIndex;
use std::path::Path;

const ZOOM_SPEED: f32 = 0.02;

// number field that shows and accepts short numbers like 1.25K
fn number_input(ui: &mut egui::Ui, id_salt: &str, value: &mut f32) -> egui::Response {
    // remmber what the user typed in the field, so we can show it again if they focus on it again
    let id = ui.make_persistent_id(id_salt);
    let text_id = id.with("text");

    let mut text = ui.data_mut(|d| {
        d.get_temp::<String>(text_id)
            .unwrap_or_else(|| conver_to_short_number(*value as f64))
    });

    let response = ui.add(egui::TextEdit::singleline(&mut text).id(id));

    if response.has_focus() {
        ui.data_mut(|d| d.insert_temp(text_id, text));
    } else {
        if response.lost_focus() {
            // try to parse the text as a number, and if it works, update the value
            if let Some(parsed) = convert_to_regular_number(&text) {
                *value = parsed as f32;
            }
            ui.data_mut(|d| d.remove::<String>(text_id));
        }
    }

    response
}

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
    scalar: f32,
    confirm_multiply: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            graph: AppGraph::build_graph(&cc.egui_ctx, &load_items(DATA_PATH)),
            editing: None,
            new_node: None,
            relayout: false,
            scalar: 1.0,
            confirm_multiply: false,
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

            ui.separator();
            egui::containers::menu::MenuButton::new("✖ Multiply")
                .config(
                    egui::containers::menu::MenuConfig::new()
                        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
                )
                .ui(ui, |ui| {
                    if !self.confirm_multiply {
                        //enter the scalar
                        ui.horizontal(|ui| {
                            ui.label("Multiply all prices by:");
                            number_input(ui, "scalar", &mut self.scalar);
                        });
                        if ui.button("Apply").clicked() {
                            self.confirm_multiply = true;
                        }
                    } else {
                        //confirm
                        ui.label(format!(
                            "Multiply every price by {}?",
                            conver_to_short_number(self.scalar as f64)
                        ));
                        ui.horizontal(|ui| {
                            if ui.button("✔ Confirm").clicked() {
                                self.graph.multiply_prices(self.scalar);
                                self.confirm_multiply = false;
                                ui.close();
                            }
                            if ui.button("Cancel").clicked() {
                                self.confirm_multiply = false;
                            }
                        });
                    }
                });
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
                    number_input(ui, "price", &mut node.payload_mut().price);
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
                        number_input(ui, "new_price", &mut form.price);
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
