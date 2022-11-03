use cpal::Device;
use cpal::traits::DeviceTrait;


fn print_supported_configs(device: &Device) {
    println!("Supported Configs:");
    let configs = device.supported_output_configs().unwrap();
    for config in configs {
        println!("\t{:?}", config);
    }
}