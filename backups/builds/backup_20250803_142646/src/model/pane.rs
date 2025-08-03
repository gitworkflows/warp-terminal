use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use iced::{Element, Border, Length};
use iced::widget::{container, row, column, text, mouse_area};
use crate::app::terminal::Message;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaneSize {
    Fixed(u16),
    Percentage(f32),
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pane {
    pub id: Uuid,
    pub title: String,
    pub is_active: bool,
    pub working_directory: std::path::PathBuf,
    pub command_history: Vec<String>,
    pub current_process: Option<String>,
    pub size: PaneSize,
    pub min_size: u16,
    pub max_size: Option<u16>,
    pub is_focused: bool,
    pub shell_type: String,
    pub env_vars: HashMap<String, String>,
}

impl Pane {
    pub fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            is_active: true,
            working_directory: std::env::current_dir().unwrap_or_default(),
            command_history: Vec::new(),
            current_process: None,
            size: PaneSize::Auto,
            min_size: 100,
            max_size: None,
            is_focused: false,
            shell_type: std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string()),
            env_vars: HashMap::new(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let header = row![
            text(&self.title).size(12),
            if self.current_process.is_some() {
                text(format!(" [{}]", self.current_process.as_ref().unwrap())).size(10)
            } else {
                text("")
            }
        ];

        let content = column![
            header,
            text(format!("Working Dir: {}", self.working_directory.display())).size(10),
            text(format!("Shell: {}", self.shell_type)).size(10),
        ];

        container(content)
            .padding(8)
            .style(if self.is_focused {
                iced::theme::Container::Custom(Box::new(FocusedPaneStyle))
            } else {
                iced::theme::Container::Custom(Box::new(UnfocusedPaneStyle))
            })
            .into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitLayout {
    pub id: Uuid,
    pub direction: SplitDirection,
    pub children: Vec<SplitNode>,
    pub active_pane: Option<Uuid>,
    
    
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplitNode {
    Pane(Pane),
    Layout(SplitLayout),
}

impl SplitLayout {
    pub fn new(direction: SplitDirection) -> Self {
        Self {
            id: Uuid::new_v4(),
            direction,
            children: Vec::new(),
            active_pane: None,
            
        }
    }

    pub fn add_pane(&mut self, pane: Pane) {
        if self.active_pane.is_none() {
            self.active_pane = Some(pane.id);
        }
        self.children.push(SplitNode::Pane(pane));
    }

    pub fn split_pane(&mut self, pane_id: Uuid, direction: SplitDirection, new_pane: Pane) -> Result<(), String> {
        Self::split_pane_recursive_static(pane_id, direction, new_pane, &mut self.children)
    }

    fn split_pane_recursive_static(pane_id: Uuid, direction: SplitDirection, new_pane: Pane, nodes: &mut Vec<SplitNode>) -> Result<(), String> {
        for i in 0..nodes.len() {
            match &mut nodes[i] {
                SplitNode::Pane(pane) if pane.id == pane_id => {
                    let existing_pane = std::mem::replace(pane, new_pane.clone());
                    let mut new_layout = SplitLayout::new(direction);
                    new_layout.add_pane(existing_pane);
                    new_layout.add_pane(new_pane);
                    nodes[i] = SplitNode::Layout(new_layout);
                    return Ok(());
                }
                SplitNode::Layout(layout) => {
                    if let Ok(()) = Self::split_pane_recursive_static(pane_id, direction, new_pane.clone(), &mut layout.children) {
                        return Ok(());
                    }
                }
                _ => {}
            }
        }
        Err("Pane not found".to_string())
    }

    pub fn close_pane(&mut self, pane_id: Uuid) -> Result<(), String> {
        SplitLayout::close_pane_recursive_static(pane_id, &mut self.children, &mut self.active_pane)
    }

    fn close_pane_recursive_static(pane_id: Uuid, nodes: &mut Vec<SplitNode>, active_pane: &mut Option<Uuid>) -> Result<(), String> {
        for i in (0..nodes.len()).rev() {
            match &mut nodes[i] {
                SplitNode::Pane(pane) if pane.id == pane_id => {
                    nodes.remove(i);
                    if *active_pane == Some(pane_id) {
                        *active_pane = Self::find_first_pane_id_in_nodes(nodes);
                    }
                    return Ok(());
                }
                SplitNode::Layout(layout) => {
                    if Self::close_pane_recursive_static(pane_id, &mut layout.children, active_pane).is_ok() {
                        if layout.children.is_empty() {
                            nodes.remove(i);
                        } else if layout.children.len() == 1 {
                            // Flatten single-child layouts
                            let child = layout.children.pop().unwrap();
                            nodes[i] = child;
                        }
                        return Ok(());
                    }
                }
                _ => {}
            }
        }
        Err("Pane not found".to_string())
    }

    fn find_first_pane_id_in_nodes(nodes: &[SplitNode]) -> Option<Uuid> {
        for node in nodes {
            match node {
                SplitNode::Pane(pane) => return Some(pane.id),
                SplitNode::Layout(layout) => {
                    if let Some(id) = layout.find_first_pane_id() {
                        return Some(id);
                    }
                }
            }
        }
        None
    }

    pub fn find_first_pane_id(&self) -> Option<Uuid> {
        for node in &self.children {
            match node {
                SplitNode::Pane(pane) => return Some(pane.id),
                SplitNode::Layout(layout) => {
                    if let Some(id) = layout.find_first_pane_id() {
                        return Some(id);
                    }
                }
            }
        }
        None
    }

    pub fn set_active_pane(&mut self, pane_id: Uuid) {
        self.active_pane = Some(pane_id);
        Self::update_focus_recursive_static(pane_id, &mut self.children);
    }

    fn update_focus_recursive_static(active_id: Uuid, nodes: &mut Vec<SplitNode>) {
        for node in nodes {
            match node {
                SplitNode::Pane(pane) => {
                    pane.is_focused = pane.id == active_id;
                }
                SplitNode::Layout(layout) => {
                    Self::update_focus_recursive_static(active_id, &mut layout.children);
                }
            }
        }
    }

    pub fn resize_pane(&mut self, pane_id_1: Uuid, pane_id_2: Uuid, delta: i16) {
        let mut pane1_idx = None;
        let mut pane2_idx = None;

        for (i, node) in self.children.iter().enumerate() {
            match node {
                SplitNode::Pane(pane) => {
                    if pane.id == pane_id_1 {
                        pane1_idx = Some(i);
                    } else if pane.id == pane_id_2 {
                        pane2_idx = Some(i);
                    }
                }
                SplitNode::Layout(layout) => {
                    // Recursively call resize on child layouts if the panes are within them
                    // This is a simplified approach and might need more complex logic for nested resizing
                    if layout.find_first_pane_id() == Some(pane_id_1) || layout.find_first_pane_id() == Some(pane_id_2) {
                        // For now, we'll assume direct children. Nested resizing will require more thought.
                    }
                }
            }
        }

        if let (Some(idx1), Some(idx2)) = (pane1_idx, pane2_idx) {
            if idx1 == idx2 { return; }

            let (node1, node2) = if idx1 < idx2 {
                let (left, right) = self.children.split_at_mut(idx2);
                (&mut left[idx1], &mut right[0])
            } else {
                let (left, right) = self.children.split_at_mut(idx1);
                (&mut right[0], &mut left[idx2])
            };

            if let (SplitNode::Pane(pane1), SplitNode::Pane(pane2)) = (node1, node2) {
                let current_size1 = match pane1.size {
                    PaneSize::Fixed(s) => s,
                    _ => 100, // Default size for auto/percentage
                };
                let min_size1 = pane1.min_size;

                let current_size2 = match pane2.size {
                    PaneSize::Fixed(s) => s,
                    _ => 100, // Default size for auto/percentage
                };
                let min_size2 = pane2.min_size;

                let new_size1 = (current_size1 as i16 + delta) as u16;
                let new_size2 = (current_size2 as i16 - delta) as u16;

                if new_size1 >= min_size1 && new_size2 >= min_size2 {
                    pane1.size = PaneSize::Fixed(new_size1);
                    pane2.size = PaneSize::Fixed(new_size2);
                }
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        self.render_nodes(&self.children)
    }

    fn render_nodes<'a>(&self, nodes: &'a [SplitNode]) -> Element<'a, Message> {
        if nodes.is_empty() {
            return text("No panes").into();
        }

        let mut elements: Vec<Element<Message>> = Vec::new();
        
        for (i, node) in nodes.iter().enumerate() {
            match node {
                SplitNode::Pane(pane) => {
                    elements.push(pane.view());
                }
                SplitNode::Layout(layout) => {
                    elements.push(layout.render_nodes(&layout.children));
                }
            }

            // Add a draggable divider
            if i < nodes.len() - 1 {
                let pane_id = match node {
                    SplitNode::Pane(p) => p.id,
                    SplitNode::Layout(l) => l.id,
                };
                let next_pane_id = match &nodes[i + 1] {
                    SplitNode::Pane(p) => p.id,
                    SplitNode::Layout(l) => l.id,
                };
                let divider = mouse_area(
                    container(text(""))
                        .width(if self.direction == SplitDirection::Vertical { Length::Fill } else { Length::Fixed(4.0) })
                        .height(if self.direction == SplitDirection::Horizontal { Length::Fill } else { Length::Fixed(4.0) })
                        .style(iced::theme::Container::Custom(Box::new(DividerStyle)))
                )
                .on_press(Message::PaneResizeStart(pane_id, next_pane_id, iced::Point::new(0.0, 0.0)))
                .on_release(Message::PaneResizeEnd)
                ;
                elements.push(divider.into());
            }
        }

        match self.direction {
            SplitDirection::Horizontal => {
                let mut r = row![];
                for element in elements {
                    r = r.push(element);
                }
                r.spacing(2).into()
            }
            SplitDirection::Vertical => {
                let mut c = column![];
                for element in elements {
                    c = c.push(element);
                }
                c.spacing(2).into()
            }
        }
    }
}

// Custom styles for panes
struct FocusedPaneStyle;
struct UnfocusedPaneStyle;
struct DividerStyle;

impl container::StyleSheet for FocusedPaneStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            border: Border {
                color: iced::Color::from_rgb(0.0, 0.5, 1.0),
                width: 2.0,
                radius: 4.0.into(),
            },
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.1, 0.1, 0.1, 0.8))),
            text_color: None,
            shadow: Default::default(),
        }
    }
}

impl container::StyleSheet for UnfocusedPaneStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            border: Border {
                color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: 4.0.into(),
            },
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.05, 0.05, 0.05, 0.8))),
            text_color: None,
            shadow: Default::default(),
        }
    }
}

impl container::StyleSheet for DividerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            border: Border::default(),
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.2, 0.2))),
            text_color: None,
            shadow: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PaneManager {
    pub root_layout: SplitLayout,
    pub pane_counter: u32,
}

impl PaneManager {
    pub fn new() -> Self {
        let mut root = SplitLayout::new(SplitDirection::Horizontal);
        let initial_pane = Pane::new("Terminal 1".to_string());
        root.add_pane(initial_pane);
        
        Self {
            root_layout: root,
            pane_counter: 1,
        }
    }

    pub fn split_current_pane(&mut self, direction: SplitDirection) -> Result<Uuid, String> {
        if let Some(active_id) = self.root_layout.active_pane {
            self.pane_counter += 1;
            let new_pane = Pane::new(format!("Terminal {}", self.pane_counter));
            let new_pane_id = new_pane.id;
            self.root_layout.split_pane(active_id, direction, new_pane)?;
            self.root_layout.set_active_pane(new_pane_id);
            Ok(new_pane_id)
        } else {
            Err("No active pane".to_string())
        }
    }

    pub fn close_current_pane(&mut self) -> Result<(), String> {
        if let Some(active_id) = self.root_layout.active_pane {
            self.root_layout.close_pane(active_id)
        } else {
            Err("No active pane".to_string())
        }
    }

    pub fn focus_next_pane(&mut self) {
        // Implementation for focusing next pane in sequence
        if let Some(next_id) = self.find_next_pane_id() {
            self.root_layout.set_active_pane(next_id);
        }
    }

    pub fn focus_previous_pane(&mut self) {
        // Implementation for focusing previous pane in sequence
        if let Some(prev_id) = self.find_previous_pane_id() {
            self.root_layout.set_active_pane(prev_id);
        }
    }

    pub fn resize_pane(&mut self, pane_id_1: Uuid, pane_id_2: Uuid, delta: i16) {
        self.root_layout.resize_pane(pane_id_1, pane_id_2, delta);
    }

    pub fn find_sibling_pane_id(&self, pane_id: Uuid, direction: SplitDirection) -> Option<Uuid> {
        self.find_sibling_pane_id_recursive(&self.root_layout.children, pane_id, direction)
    }

    fn find_sibling_pane_id_recursive(&self, nodes: &[SplitNode], target_pane_id: Uuid, direction: SplitDirection) -> Option<Uuid> {
        for i in 0..nodes.len() {
            match &nodes[i] {
                SplitNode::Pane(pane) => {
                    if pane.id == target_pane_id {
                        // Found the target pane, now find its sibling
                        if direction == SplitDirection::Horizontal {
                            // Looking for sibling to the right
                            if i + 1 < nodes.len() {
                                return self.get_first_pane_id_in_node(&nodes[i + 1]);
                            }
                        } else { // Vertical
                            // Looking for sibling below
                            if i + 1 < nodes.len() {
                                return self.get_first_pane_id_in_node(&nodes[i + 1]);
                            }
                        }
                    }
                }
                SplitNode::Layout(layout) => {
                    // If the target pane is within a nested layout, recurse into it
                    if let Some(sibling_id) = self.find_sibling_pane_id_recursive(&layout.children, target_pane_id, direction) {
                        return Some(sibling_id);
                    }
                }
            }
        }
        None
    }

    fn get_first_pane_id_in_node(&self, node: &SplitNode) -> Option<Uuid> {
        match node {
            SplitNode::Pane(pane) => Some(pane.id),
            SplitNode::Layout(layout) => layout.find_first_pane_id(),
        }
    }

    fn find_next_pane_id(&self) -> Option<Uuid> {
        let pane_ids = self.collect_all_pane_ids();
        if let Some(current_idx) = self.root_layout.active_pane
            .and_then(|id| pane_ids.iter().position(|&pid| pid == id)) {
            let next_idx = (current_idx + 1) % pane_ids.len();
            pane_ids.get(next_idx).copied()
        } else {
            pane_ids.first().copied()
        }
    }

    fn find_previous_pane_id(&self) -> Option<Uuid> {
        let pane_ids = self.collect_all_pane_ids();
        if let Some(current_idx) = self.root_layout.active_pane
            .and_then(|id| pane_ids.iter().position(|&pid| pid == id)) {
            let prev_idx = if current_idx == 0 { pane_ids.len() - 1 } else { current_idx - 1 };
            pane_ids.get(prev_idx).copied()
        } else {
            pane_ids.first().copied()
        }
    }

    fn collect_all_pane_ids(&self) -> Vec<Uuid> {
        let mut ids = Vec::new();
        self.collect_pane_ids_recursive(&self.root_layout.children, &mut ids);
        ids
    }

    fn collect_pane_ids_recursive(&self, nodes: &[SplitNode], ids: &mut Vec<Uuid>) {
        for node in nodes {
            match node {
                SplitNode::Pane(pane) => ids.push(pane.id),
                SplitNode::Layout(layout) => self.collect_pane_ids_recursive(&layout.children, ids),
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        self.root_layout.view()
    }
}

impl Default for PaneManager {
    fn default() -> Self {
        Self::new()
    }
}
