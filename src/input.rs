pub type HashSet<K   > = wgpu::naga::FastHashSet<K   >;
pub type HashMap<K, V> = wgpu::naga::FastHashMap<K, V>;

macro_rules! count {
    ($($x:tt)*) => {
        <[()]>::len(&[$( { let _ = stringify!($x); () } ),*])
    };
}


macro_rules! enum_with_variant_count {
    ($vis: vis enum $name: ident {
        $($variant: ident),*
    }) => {
        #[repr(u32)]
        #[derive(Debug, Clone, Copy, Eq, PartialEq)]
        $vis enum $name {
            $($variant),*
        }
        impl $name {
            $vis const VARIANT_COUNT: u32 = count!($($variant)*) as u32;
        }
    };
}

enum_with_variant_count! { 
    pub enum ActionBinding {
        MoveForward,
        MoveBack,
        MoveRight,
        MoveLeft,
        MoveUp,
        MoveDown,
        MouseCapture
    }
}

macro_rules! hashmap {
    ($($($key: expr),+ => $value: expr;)*) => {{
        let mut h = HashMap::default();
        $($(h.insert($key, $value);)+)*
        h
    }};
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum KeyState {
    JustPressed,
    Pressed,
    JustReleased,
    #[default]
    Released,
}

#[derive(Debug, Clone)]
pub struct InputBinder {
    pub mouse_sensitivity: glam::DVec2,
    pub key_bindings: HashMap<winit::keyboard::PhysicalKey, ActionBinding>,
    
    pub mouse_motion: glam::DVec2,
    pub actions: [KeyState; ActionBinding::VARIANT_COUNT as usize],
}

impl Default for InputBinder {
    fn default() -> Self {
        Self {
            mouse_sensitivity: glam::DVec2::splat(0.006),
            key_bindings: default_keybindings(),
            mouse_motion: glam::DVec2::ZERO,
            actions: Default::default(),
        }
    }
}

pub fn default_keybindings() -> HashMap<winit::keyboard::PhysicalKey, ActionBinding> {
    use winit::keyboard::{PhysicalKey::Code, KeyCode};

    hashmap! { 
        Code(KeyCode::KeyW), Code(KeyCode::ArrowUp)    => ActionBinding::MoveForward;
        Code(KeyCode::KeyS), Code(KeyCode::ArrowDown)  => ActionBinding::MoveBack;
        Code(KeyCode::KeyA), Code(KeyCode::ArrowLeft)  => ActionBinding::MoveLeft;
        Code(KeyCode::KeyD), Code(KeyCode::ArrowRight) => ActionBinding::MoveRight;
        Code(KeyCode::KeyQ)                            => ActionBinding::MoveDown;
        Code(KeyCode::KeyE)                            => ActionBinding::MoveUp;
        Code(KeyCode::Tab)                             => ActionBinding::MouseCapture;
    }
}

impl InputBinder {
    pub fn process_device_event(
        &mut self, 
        _device_id: winit::event::DeviceId, 
        event: winit::event::DeviceEvent, 
    ) {
        use winit::event::DeviceEvent::*;
        use winit::event::RawKeyEvent;
        match event {
            MouseMotion { delta: (dx, dy) } => {
                self.mouse_motion = glam::DVec2::new(dx, dy);
            }
            Key(RawKeyEvent { physical_key, state }) => {
                if let Some(&binding) = self.key_bindings.get(&physical_key) {
                    if state.is_pressed() {
                        self.actions[binding as usize] = KeyState::JustPressed;
                    } else {
                        self.actions[binding as usize] = KeyState::JustReleased;
                    }
                } else {
                    log::debug!("Attempted to process Unset Keybinding: {physical_key:?}.");
                }
            }
            
            _ => {}
        }
    }

    pub fn end_frame(&mut self) {
        self.mouse_motion = glam::DVec2::ZERO;
        self.actions = self.actions.map(|state| match state {
            KeyState::JustPressed => KeyState::Pressed,
            KeyState::Pressed => KeyState::Pressed,
            KeyState::JustReleased => KeyState::Released,
            KeyState::Released => KeyState::Released,
        });
    }

    pub fn mouse_motion(&self) -> glam::DVec2 {
        self.mouse_motion * self.mouse_sensitivity
    }
}

impl std::ops::Index<ActionBinding> for InputBinder {
    type Output = KeyState;

    fn index(&self, index: ActionBinding) -> &Self::Output {
        &self.actions[index as usize]
    }
}

impl KeyState {
    pub fn is_pressed(self) -> bool {
        match self {
            KeyState::JustPressed => true,
            KeyState::Pressed => true,
            KeyState::JustReleased => false,
            KeyState::Released => false,
        }
    }
}

impl Into<winit::event::ElementState> for KeyState {
    fn into(self) -> winit::event::ElementState {
        match self {
            KeyState::JustPressed => winit::event::ElementState::Pressed,
            KeyState::Pressed => winit::event::ElementState::Pressed,
            KeyState::JustReleased => winit::event::ElementState::Released,
            KeyState::Released => winit::event::ElementState::Released,
        }
    }
}

impl From<winit::event::ElementState> for KeyState {
    fn from(state: winit::event::ElementState) -> KeyState {
        match state {
            winit::event::ElementState::Pressed => KeyState::Pressed,
            winit::event::ElementState::Released => KeyState::Released,
        }
    }
}