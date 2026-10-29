use egui::{Color32, Pos2, Rect, Shape, TextureHandle, Vec2};
use egui_graphs::{DisplayNode, DrawContext, NodeProps};
use petgraph::EdgeType;
use petgraph::prelude::NodeIndex;
use petgraph::stable_graph::IndexType;

pub const NODE_SIZE: f32 = 25.0;

//what we store in the graph node
#[derive(Clone)]
pub struct NodeData {
    pub texture: TextureHandle,
    pub image: Option<String>, // the path from the JSON
    pub price: f32,
    pub children: Vec<NodeIndex>,
}

//how a node is drawn
#[derive(Clone)]
pub struct ImageNode {
    location: Pos2,
    texture: TextureHandle,
}

impl From<NodeProps<NodeData>> for ImageNode {
    fn from(props: NodeProps<NodeData>) -> Self {
        Self {
            location: props.location(),
            texture: props.payload.texture.clone(),
        }
    }
}

impl<E: Clone, Ty: EdgeType, Ix: IndexType> DisplayNode<NodeData, E, Ty, Ix> for ImageNode {
    fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
        self.location + dir.normalized() * (NODE_SIZE / 2.0)
    }

    fn is_inside(&self, pos: Pos2) -> bool {
        Rect::from_center_size(self.location, Vec2::splat(NODE_SIZE)).contains(pos)
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
