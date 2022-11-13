use sysinfo::{CpuExt,System, SystemExt};
use common_math::rounding;
use chrono::prelude::*;
use configparser::ini::Ini;
use std::error::Error;
use serial2::SerialPort;

fn main() -> Result<(),Box<dyn Error>> {
    //Get settings
    let mut config = Ini::new();
    config.load("target/debug/config.ini")?;
    let config_sends_per_second:u64=config.getuint("transfer", "sends per second").unwrap().unwrap();
    let config_serial_device:String=config.get("transfer", "serial device").unwrap();
    
    //Init serial port
    let port=SerialPort::open(config_serial_device, 9600)?;
    
    //Init system data
    let mut sys=System::new_all();

    loop {
        let mut dt=Local::now();

        sys.refresh_cpu();

        let cpu_percentage:u8;
        let memory_percentage:u8;

        let mut cpu_cores:u8=0;
        let mut percentage_per_core:f32=0.0;

        for cpu in sys.cpus(){
            cpu_cores+=1;
            percentage_per_core+=cpu.cpu_usage();
        }
        cpu_percentage=rounding::floor(percentage_per_core/cpu_cores as f32,0) as u8;

        sys.refresh_memory();
        memory_percentage=rounding::floor(((sys.used_memory()/1048576) as f64 / (sys.total_memory()/1048576) as f64)*100.0,0) as u8;

        //Write to serial

        let string_to_send = format!("<hi,{},{},{},{}>",&cpu_percentage.to_string(),
                                                                &memory_percentage.to_string(),
                                                                dt.format("%H").to_string(),
                                                                dt.format("%M").to_string());

        
        port.write(string_to_send.as_bytes())?;
        //println!("{:?}",string_to_send);

        std::thread::sleep(std::time::Duration::from_millis(1000/config_sends_per_second));
    }
}
