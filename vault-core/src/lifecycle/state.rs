#[derive(Debug, Clone)]
pub enum AppVisibility {
    Visible,
    Hidden,
}

impl Default for AppVisibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(Debug, Clone, Default)]
pub struct LifecycleState {
    pub app_visibility: AppVisibility,
}

impl LifecycleState {
    pub fn reset(&mut self) {
        *self = Default::default()
    }
}
