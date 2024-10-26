use serde::Deserialize;

use super::pid_controller::PIDParameters;

#[derive(Debug, Deserialize)]
pub struct ElevatorParameters {
    pub pid_parameters: PIDParameters,
    pub floors: Vec<f32>,
    pub max_speed: f32,
    pub max_accel: f32,
    pub max_load: f32,
    pub elevator_mass: f32,
    pub elevator_counter_mass: f32,

    #[serde(default = "default_enable_debug_plotting")]
    pub enable_debug_plotting: bool,
    #[serde(default = "default_plot_path")]
    pub plot_path: String,

    //      bu current değişkenler normalde gerekli değil çünkü 
    // sıfırdan başlıyoruz ammavelakin olur da asansör sisteminde 
    // bir sıkıntı çıkarsa ve program kendini yeniden başlatırsa, 
    // herkes ölmesin diye bunları ekledim. boğaziçi yarışmasına 
    // kadar muhtemelen kaldığı yerden devam ettirme özelliğini
    // ekleyemem ama daha sonrası için kolaylık olur.

    // #[serde(default = "default_current_height")]
    // pub current_height: f32,
    // #[serde(default = "default_current_load")]
    // pub current_load: f32,
    // #[serde(default = "default_current_accelaration")]
    // pub current_acceleration: f32,
    // #[serde(default = "default_enable_debug_plotting")]
    // pub current_speed: f32,
}

fn default_enable_debug_plotting() -> bool { false }
fn default_plot_path() -> String { "motor_plot.png".to_string() }