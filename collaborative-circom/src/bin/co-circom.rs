use std::path::PathBuf;

use clap::{Parser, Subcommand};
use collaborative_circom::{config::NetworkConfig, file_utils};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Splits an existing witness file generated by Circom into secret shares for use in MPC
    SplitWitness {
        /// The path to the input witness file generated by Circom
        #[arg(long)]
        input: PathBuf,
        /// The MPC protocol to be used
        #[arg(long)]
        protocol: String, // TODO: which datatype? an enum?
        /// The path to the (existing) output directory
        #[arg(long)]
        out_dir: PathBuf,
    },
    /// Splits a JSON input file into secret shares for use in MPC
    SplitInput {
        /// The path to the input JSON file
        #[arg(long)]
        input: PathBuf,
        /// The MPC protocol to be used
        #[arg(long)]
        protocol: String, // TODO: which datatype? an enum?
        /// The path to the (existing) output directory
        #[arg(long)]
        out_dir: PathBuf,
    },
    /// Evaluates the extended witness generation for the specified circuit and input share in MPC
    GenerateWitness {
        /// The path to the input share file
        #[arg(long)]
        input: PathBuf,
        /// The path to the circuit file
        #[arg(long)]
        circuit: PathBuf,
        /// The MPC protocol to be used
        #[arg(long)]
        protocol: String, // TODO: which datatype? an enum?
        /// The path to MPC network configuration file
        #[arg(long)]
        config: PathBuf,
        /// The output file where the final witness share is written to
        #[arg(long)]
        out: PathBuf,
    },
    /// Evaluates the prover algorithm for the specified circuit and witness share in MPC
    GenerateProof {
        /// The path to the witness share file
        #[arg(long)]
        witness: PathBuf,
        /// The path to the r1cs file, generated by Circom compiler
        #[arg(long)]
        r1cs: PathBuf,
        /// The path to the proving key (.zkey) file, generated by snarkjs setup phase
        #[arg(long)]
        zkey: PathBuf,
        /// The MPC protocol to be used
        #[arg(long)]
        protocol: String, // TODO: which datatype? an enum?
        /// The path to MPC network configuration file
        #[arg(long)]
        config: PathBuf,
        /// The output file where the final proof is written to
        #[arg(long)]
        out: PathBuf,
    },
}

fn main() -> color_eyre::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::SplitWitness {
            input,
            protocol,
            out_dir,
        } => {
            file_utils::check_file_exists(&input)?;
            file_utils::check_dir_exists(&out_dir)?;

            // read the Circom witness file

            // construct relevant protocol

            // create witness shares

            // write out the shares to the output directory
        }
        Commands::SplitInput {
            input,
            protocol,
            out_dir,
        } => {
            file_utils::check_file_exists(&input)?;
            file_utils::check_dir_exists(&out_dir)?;

            // read the input file

            // construct relevant protocol

            // create input shares

            // write out the shares to the output directory
        }
        Commands::GenerateWitness {
            input,
            circuit,
            protocol,
            config,
            out,
        } => {
            file_utils::check_file_exists(&input)?;
            file_utils::check_file_exists(&circuit)?;
            file_utils::check_file_exists(&config)?;

            // parse input shares

            // parse circuit file & put through our compiler

            // parse network configuration
            let config = std::fs::read_to_string(config)?;
            let config: NetworkConfig = toml::from_str(&config)?;

            // construct relevant protocol

            // connect to network

            // execute witness generation in MPC

            // write result to output file
            let out_file = std::fs::File::create(out)?;
        }
        Commands::GenerateProof {
            witness,
            r1cs,
            zkey,
            protocol,
            config,
            out,
        } => {
            file_utils::check_file_exists(&witness)?;
            file_utils::check_file_exists(&r1cs)?;
            file_utils::check_file_exists(&zkey)?;
            file_utils::check_file_exists(&config)?;

            // parse witness shares

            // parse Circom r1cs file

            // parse Circom zkey file

            // parse network configuration
            let config = std::fs::read_to_string(config)?;
            let config: NetworkConfig = toml::from_str(&config)?;

            // construct relevant protocol

            // connect to network

            // execute prover in MPC

            // write result to output file
            let out_file = std::fs::File::create(out)?;
        }
    }

    Ok(())
}
