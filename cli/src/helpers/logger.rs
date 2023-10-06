// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkOS library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::helpers::LogWriter;

use crossterm::tty::IsTty;
use std::{fs::File, io, path::Path};
use tokio::sync::mpsc;
use tracing_subscriber::{
    layer::{Layer, SubscriberExt},
    util::SubscriberInitExt,
    EnvFilter,
};

/// Initializes the logger.
///
/// ```ignore
/// 0 => info
/// 1 => info, debug
/// 2 => info, debug, trace
/// 3 => info, debug, trace, snarkos_node_narwhal::gateway=trace
/// 4 => info, debug, trace, snarkos_node_narwhal=trace
/// 5 => info, debug, trace, snarkos_node_router=trace
/// 6 => info, debug, trace, snarkos_node_tcp=trace
/// ```
pub fn initialize_logger<P: AsRef<Path>>(verbosity: u8, nodisplay: bool, logfile: P) -> mpsc::Receiver<Vec<u8>> {
    match verbosity {
        0 => std::env::set_var("RUST_LOG", "info"),
        1 => std::env::set_var("RUST_LOG", "debug"),
        2.. => std::env::set_var("RUST_LOG", "trace"),
    };

    // Filter out undesirable logs. (unfortunately EnvFilter cannot be cloned)
    let [filter, filter2] = std::array::from_fn(|_| {
        let filter = EnvFilter::from_default_env()
            .add_directive("mio=off".parse().unwrap())
            .add_directive("tokio_util=off".parse().unwrap())
            .add_directive("hyper=off".parse().unwrap())
            .add_directive("reqwest=off".parse().unwrap())
            .add_directive("want=off".parse().unwrap())
            .add_directive("warp=off".parse().unwrap());

        let filter = if verbosity > 3 {
            filter.add_directive("snarkos_node_narwhal::gateway=trace".parse().unwrap())
        } else {
            filter.add_directive("snarkos_node_narwhal::gateway=debug".parse().unwrap())
        };

        let filter = if verbosity > 4 {
            filter.add_directive("snarkos_node_narwhal=trace".parse().unwrap())
        } else {
            filter.add_directive("snarkos_node_narwhal=debug".parse().unwrap())
        };

        let filter = if verbosity > 5 {
            filter.add_directive("snarkos_node_router=trace".parse().unwrap())
        } else {
            filter.add_directive("snarkos_node_router=off".parse().unwrap())
        };

        if verbosity > 6 {
            filter.add_directive("snarkos_node_tcp=trace".parse().unwrap())
        } else {
            filter.add_directive("snarkos_node_tcp=off".parse().unwrap())
        }
    });

    // Create the directories tree for a logfile if it doesn't exist.
    let logfile_dir = logfile.as_ref().parent().expect("Root directory passed as a logfile");
    if !logfile_dir.exists() {
        std::fs::create_dir_all(logfile_dir)
            .expect("Failed to create a directories: '{logfile_dir}', please check if user has permissions");
    }
    // Create a file to write logs to.
    let logfile =
        File::options().append(true).create(true).open(logfile).expect("Failed to open the file for writing logs");

    // Initialize the log channel.
    let (log_sender, log_receiver) = mpsc::channel(1024);

    // Initialize the log sender.
    let log_sender = match nodisplay {
        true => None,
        false => Some(log_sender),
    };

    // Initialize tracing.
    let _ = tracing_subscriber::registry()
        .with(
            // Add layer using LogWriter for stdout / terminal
            tracing_subscriber::fmt::Layer::default()
                .with_ansi(log_sender.is_none() && io::stdout().is_tty())
                .with_writer(move || LogWriter::new(&log_sender))
                .with_target(verbosity > 2)
                .with_filter(filter),
        )
        .with(
            // Add layer redirecting logs to the file
            tracing_subscriber::fmt::Layer::default()
                .with_ansi(false)
                .with_writer(logfile)
                .with_target(verbosity > 2)
                .with_filter(filter2),
        )
        .try_init();

    log_receiver
}

/// Returns the welcome message as a string.
pub fn welcome_message() -> String {
    use colored::Colorize;

    let mut output = String::new();
    output += &r#"

         ╦╬╬╬╬╬╦
        ╬╬╬╬╬╬╬╬╬                    ▄▄▄▄        ▄▄▄
       ╬╬╬╬╬╬╬╬╬╬╬                  ▐▓▓▓▓▌       ▓▓▓
      ╬╬╬╬╬╬╬╬╬╬╬╬╬                ▐▓▓▓▓▓▓▌      ▓▓▓     ▄▄▄▄▄▄       ▄▄▄▄▄▄
     ╬╬╬╬╬╬╬╬╬╬╬╬╬╬╬              ▐▓▓▓  ▓▓▓▌     ▓▓▓   ▄▓▓▀▀▀▀▓▓▄   ▐▓▓▓▓▓▓▓▓▌
    ╬╬╬╬╬╬╬╜ ╙╬╬╬╬╬╬╬            ▐▓▓▓▌  ▐▓▓▓▌    ▓▓▓  ▐▓▓▓▄▄▄▄▓▓▓▌ ▐▓▓▓    ▓▓▓▌
   ╬╬╬╬╬╬╣     ╠╬╬╬╬╬╬           ▓▓▓▓▓▓▓▓▓▓▓▓    ▓▓▓  ▐▓▓▀▀▀▀▀▀▀▀▘ ▐▓▓▓    ▓▓▓▌
  ╬╬╬╬╬╬╣       ╠╬╬╬╬╬╬         ▓▓▓▓▌    ▐▓▓▓▓   ▓▓▓   ▀▓▓▄▄▄▄▓▓▀   ▐▓▓▓▓▓▓▓▓▌
 ╬╬╬╬╬╬╣         ╠╬╬╬╬╬╬       ▝▀▀▀▀      ▀▀▀▀▘  ▀▀▀     ▀▀▀▀▀▀       ▀▀▀▀▀▀
╚╬╬╬╬╬╩           ╩╬╬╬╬╩


"#
    .white()
    .bold();
    output += &"👋 Welcome to Aleo! We thank you for running a node and supporting privacy.\n".bold();
    output
}

/// Returns the notification message as a string.
pub fn notification_message() -> String {
    use colored::Colorize;

    let mut output = String::new();
    output += &r#"

 ==================================================================================================

                     🚧 Welcome to Aleo Testnet 3 Phase 3 - Calibration Period 🚧

 ==================================================================================================

     During the calibration period, the network will be running in limited capacity.

     This calibration period is to ensure validators are stable and ready for mainnet launch.
     During this period, the objective is to assess, adjust, and align validators' performance,
     stability, and interoperability under varying network conditions.

     There will be possibly several network resets. With each network reset, software updates will
     be performed to address potential bottlenecks, vulnerabilities, and/or inefficiencies, which
     will ensure optimal performance for the ecosystem of validators, provers, and developers.

 ==================================================================================================

    Duration:
    - Start Date: September 27, 2023
    - End Date: October 18, 2023 (subject to change)

    Participation:
    - Node operators are NOT REQUIRED to participate during this calibration period.

    Network Resets:
    - IMPORTANT: EXPECT MULTIPLE NETWORK RESETS.
    - If participating, BE PREPARED TO RESET YOUR NODE AT ANY TIME.

    Communication:
    - Stay ONLINE and MONITOR our Discord and Twitter for community updates.

    Purpose:
    - This period is STRICTLY FOR NETWORK CALIBRATION.
    - This period is NOT INTENDED for general-purpose usage by developers and provers.

    Incentives:
    - There are NO INCENTIVES during this calibration period.

 ==================================================================================================
"#
    .white()
    .bold();

    output
}
