use winit::window::CursorIcon;


#[derive(Debug, Default, Clone, Copy)]
pub enum Action {
    Channel {
        channel_action: ChannelAction,
        channel_index: usize,
        channel_location: u64
    },
    #[default] NoAction
}

#[derive(Debug, Clone, Copy)]
pub enum ChannelAction {
    GrabClip {
        clip_index: usize
    },
    CreateJunction,
    ModifyJunction,
    SetPlayhead
}

#[derive(Debug, Clone, Copy)]
pub enum State {
    GrabbingClip {
        channel_index: usize,
        clip_index: usize,
        relative_location: u64
    },
    CreatingJunction {
        source_channel_index: usize,
        source_channel_location: u64
    },
    Hovering {
        potential_action: Action
    },
}

impl Default for State {
    fn default() -> Self {
        State::Hovering { potential_action: Action::NoAction }
    }
}

impl State {
    pub fn cursor_icon(&self) -> CursorIcon {
        match self {
            State::GrabbingClip { .. } => CursorIcon::Grabbing,
            State::CreatingJunction { .. } => CursorIcon::Hand,
            State::Hovering { potential_action } => {
                match potential_action {
                    Action::Channel { channel_action: action, .. } => {
                        match action {
                            ChannelAction::GrabClip { .. } => CursorIcon::Grab,
                            ChannelAction::CreateJunction => CursorIcon::Hand,
                            ChannelAction::ModifyJunction => CursorIcon::Default,
                            ChannelAction::SetPlayhead => CursorIcon::Crosshair,
                        }
                    },
                    Action::NoAction => CursorIcon::Default
                }
            }
        }
    }
}
