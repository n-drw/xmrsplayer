use clap::Parser;
use console::{Key, Term};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

use xmrs::amiga::amiga_module::AmigaModule;
use xmrs::prelude::*;
use xmrs::s3m::s3m_module::S3mModule;
use xmrs::xm::xmmodule::XmModule;

use xmrsplayer::prelude::*;

#[cfg(feature = "sid")]
use xmrs::sid::sid_module::SidModule;

#[derive(Parser)]
struct Cli {
    /// Choose XM or XmRs File
    #[arg(short = 'f', long, required = true, value_name = "filename")]
    filename: Option<String>,

    /// Choose output wave file
    #[arg(short = 'o', long, value_name = "output filename")]
    output: Option<String>,

    /// Choose amplification
    #[arg(short = 'a', long, default_value = "10.0")]
    amplification: f32,

    /// Play only a specific channel (from 1 to n, 0 for all)
    #[arg(short = 'c', long, default_value = "0")]
    ch: u8,

    /// Turn debugging information on
    #[arg(short = 'd', long, default_value = "false")]
    debug: bool,

    /// How many loop (default: infinity)
    #[arg(short = 'l', long, default_value = "0")]
    loops: usize,

    /// Force historical ft2 replay (default: autodetect)
    #[arg(short = 't', long, default_value = "false")]
    historical: bool,

    /// Start at a specific pattern order table position
    #[arg(short = 'p', long, default_value = "0")]
    position: usize,

    /// Force speed
    #[arg(short = 's', long, default_value = "0")]
    speed: u16,

    /// Test SID player as a Proof of Concept
    #[cfg(feature = "sid")]
    #[arg(short = 'z', long, default_value = "false")]
    sid_test_player: bool,
}

#[cfg(feature = "sid")]
fn sid_test_player(cli: &Cli) {
    // let sidmodule = SidModule::get_sid_commando();
    // let sidmodule = SidModule::get_sid_crazy_comets();
    let sidmodule = SidModule::get_sid_monty_on_the_run();
    // let sidmodule = SidModule::get_sid_last_v8();
    // let sidmodule = SidModule::get_sid_thing_on_a_spring();
    // let sidmodule = SidModule::get_sid_zoid();
    // let sidmodule = SidModule::get_sid_ace_2();
    // let sidmodule = SidModule::get_sid_delta();
    // let sidmodule = SidModule::get_sid_human_race();
    // let sidmodule = SidModule::get_sid_international_karate();
    // let sidmodule = SidModule::get_sid_lightforce();
    // let sidmodule = SidModule::get_sid_sanxion_song_1();
    // let sidmodule = SidModule::get_sid_sanxion_song_2();
    // let sidmodule = SidModule::get_sid_spellbound();

    let modules = sidmodule.to_modules(false);

    let leaked_modules: &'static [Module] = Box::leak(modules.into_boxed_slice());
    let module_ref: &'static Module = &leaked_modules[0];

    play_music(
        module_ref,
        cli.amplification,
        cli.position,
        cli.loops,
        cli.debug,
        cli.ch,
        cli.speed,
        false,
        cli.output.clone(),
    );
}

fn main() -> Result<(), std::io::Error> {
    let cli = Cli::parse();

    // Term::stdout().clear_screen().unwrap();
    println!("--===~ XmRs Player Example ~===--");
    println!("(c) 2023-2024 Sébastien Béchet\n");
    println!("Because demo scene can't die :)\n");

    // Ugly Hack just for fun
    #[cfg(feature = "sid")]
    if cli.sid_test_player {
        sid_test_player(&cli);
        return Ok(());
    }

    match cli.filename {
        Some(filename) => {
            println!("opening {}", filename);
            let contents = std::fs::read(filename.trim())?;
            match filename.split('.').last() {
                Some(extension) if extension == "xm" || extension == "XM" => {
                    match XmModule::load(&contents) {
                        Ok(xm) => {
                            drop(contents); // cleanup memory
                            let module = xm.to_module();
                            drop(xm);
                            println!("Playing {} !", module.name);

                            let module = Box::new(module);
                            let module_ref: &'static Module = Box::leak(module);
                            play_music(
                                module_ref,
                                cli.amplification,
                                cli.position,
                                cli.loops,
                                cli.debug,
                                cli.ch,
                                cli.speed,
                                cli.historical,
                                cli.output.clone(),
                            );
                        }
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }
                }
                Some(extension) if extension == "mod" || extension == "MOD" => {
                    match AmigaModule::load(&contents) {
                        Ok(amiga) => {
                            drop(contents); // cleanup memory
                            let module = amiga.to_module();
                            drop(amiga);
                            println!("Playing {} !", module.name);
                            let module = Box::new(module);
                            let module_ref: &'static Module = Box::leak(module);
                            play_music(
                                module_ref,
                                cli.amplification,
                                cli.position,
                                cli.loops,
                                cli.debug,
                                cli.ch,
                                cli.speed,
                                false,
                                cli.output.clone(),
                            );
                        }
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }
                }
                Some(extension) if extension == "s3m" || extension == "S3M" => {
                    match S3mModule::load(&contents) {
                        Ok(s3m) => {
                            drop(contents); // cleanup memory
                            let module = s3m.to_module();
                            drop(s3m);
                            println!("Playing {} !", module.name);
                            let module = Box::new(module);
                            let module_ref: &'static Module = Box::leak(module);
                            play_music(
                                module_ref,
                                cli.amplification,
                                cli.position,
                                cli.loops,
                                cli.debug,
                                cli.ch,
                                cli.speed,
                                false,
                                cli.output.clone(),
                            );
                        }
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }
                }
                Some(_) | None => {
                    println!("File unknown?");
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn play_music(
    module: &'static Module,
    amplification: f32,
    position: usize,
    loops: usize,
    debug: bool,
    ch: u8,
    speed: u16,
    historical: bool,
    output: Option<String>,
) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");

    let config = device
        .default_output_config()
        .expect("failed to get default output config");
    let sample_rate = config.sample_rate();
    // try to detect FT2 to play historical bugs
    let is_ft2 = historical
        || module.comment == "FastTracker v2.00 (1.02)"
        || module.comment == "FastTracker v2.00 (1.03)"
        || module.comment == "FastTracker v2.00 (1.04)";

    let player = Arc::new(Mutex::new(XmrsPlayer::new(
        module,
        sample_rate.0 as f32,
        is_ft2,
    )));

    {
        let mut player_lock = player.lock().unwrap();
        player_lock.amplification = amplification;
        if debug {
            println!("Debug on");
            if is_ft2 {
                println!("FT2 Historical XM detected.")
            }
        }
        player_lock.debug(debug);
        if ch != 0 {
            player_lock.mute_all(true);
            player_lock.set_mute_channel((ch - 1).into(), false);
        }
        player_lock.set_max_loop_count(loops);
        player_lock.goto(position, 0, speed);
    }

    if let Some(output) = output {
        println!("writing {}...", output);
        write_wave(player, output.as_str()).unwrap();
    } else {
        let player_clone = Arc::clone(&player);
        let stream = device
            .build_output_stream(
                &config.config(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut player_lock = player_clone.lock().unwrap();
                    data.iter_mut()
                        .zip(player_lock.by_ref()) // itère sur les deux en parallèle
                        .for_each(|(sample, value)| *sample = value);
                },
                |_: cpal::StreamError| {},
                None,
            )
            .expect("failed to build output stream");

        stream.play().expect("failed to play stream");

        let stdout = Term::stdout();
        println!(
            "Enter and i keys for info, Space for pause, left or right arrow to move, escape key to exit..."
        );
        let mut playing = true;
        loop {
            if let Ok(character) = stdout.read_key() {
                match character {
                    Key::Enter => {
                        let ti = player.lock().unwrap().get_current_table_index();
                        let p = player.lock().unwrap().get_current_pattern();
                        println!("current table index:{:02x}, current pattern:{:02x}", ti, p);
                    }
                    Key::Escape => {
                        println!("Have a nice day!");
                        return;
                    }
                    Key::Char('q') => {
                        println!("Have a nice day!");
                        return;
                    }
                    Key::ArrowLeft => {
                        let i = player.lock().unwrap().get_current_table_index();
                        if i != 0 {
                            player.lock().unwrap().goto(i - 1, 0, 0);
                        }
                    }
                    Key::ArrowRight => {
                        let len = module.pattern_order.len();
                        let i = player.lock().unwrap().get_current_table_index();
                        if i + 1 < len {
                            player.lock().unwrap().goto(i + 1, 0, 0);
                        }
                    }
                    Key::Char(' ') => {
                        if playing {
                            println!("Pause, press space to continue");
                            player.lock().unwrap().pause(true);
                            playing = false;
                            {
                                let player_lock = player.lock().unwrap();
                                let ti = player_lock.get_current_table_index();
                                let p = player_lock.get_current_pattern();
                                let row = player_lock.get_current_row();
                                println!("Pattern [{:02X}]={:02X}, Row {:02X}", ti, p, row);
                            }
                        } else {
                            println!("Playing");
                            player.lock().unwrap().pause(false);
                            playing = true;
                        }
                    }
                    Key::Char('i') => {
                        let player_lock = player.lock().unwrap();
                        println!(
                            "name:{}\ncomment:{}",
                            player_lock.module.name, player_lock.module.comment
                        );
                        println!(
                            "speed={}, generated samples:{}, loop count:{}",
                            player_lock.get_tempo(),
                            player_lock.generated_samples,
                            player_lock.get_loop_count()
                        );
                        for (i, instr) in player_lock.module.instrument.iter().enumerate() {
                            if instr.name != "" {
                                println!("instrument {:2}: {}", i, instr.name);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

use hound::{SampleFormat, WavSpec, WavWriter};

fn write_wave(
    amp: Arc<Mutex<XmrsPlayer>>,
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let spec = WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(output_file, spec)?;
    amp.lock().unwrap().set_max_loop_count(1);
    let player_clone = Arc::clone(&amp);
    let mut player_lock = player_clone.lock().unwrap();

    for sample in player_lock.by_ref() {
        let sample_i16 = (sample * 32767.0).round() as i16;
        writer.write_sample(sample_i16)?;
    }

    writer.finalize()?;
    Ok(())
}
