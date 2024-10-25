pub struct LinePlotter {
    pub enable_plotting: bool,

} 

impl LinePlotter {
    pub fn new(enable_plotting: bool) -> Self {
        Self {
            enable_plotting,
        }
    }

    pub fn create_plot(&self) {
        // create a plot for the motor
        // this function is not implemented yet
    }

    pub fn update_plot(&self) {

    }
}