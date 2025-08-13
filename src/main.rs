use std::{env, fs::File, io::Write, path::Path, process::Command};

use clap::Parser;
// This program is used to extract build parameters from the Ledger C SDK
// It runs the `make --trace --dry-run` command and processes the output to extract
// the defines and cflags used in the build process.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the application to build
    #[arg(short, long)]
    app_path: String,

    #[arg(short, long)]
    device: String,
}

const FILTERED_DEFINES: [&str; 67] = [
    "APPNAME",
    "HAVE_SWAP",
    "PRINTF\\(...\\)",
    "MAJOR_VERSION",
    "MINOR_VERSION",
    "PATCH_VERSION",
    "API_LEVEL",
    "TARGET",
    "TARGET_NAME",
    "APPVERSION",
    "SDK_NAME",
    "SDK_VERSION",
    "SDK_HASH",
    "HAVE_NES_CRYPT",
    "HAVE_ST_AES",
    "NATIVE_LITTLE_ENDIAN",
    "HAVE_CRC",
    "HAVE_HASH",
    "HAVE_RIPEMD160",
    "HAVE_SHA224",
    "HAVE_SHA256",
    "HAVE_SHA3",
    "HAVE_SHA384",
    "HAVE_SHA512",
    "HAVE_SHA512_WITH_BLOCK_ALT_METHOD",
    "HAVE_SHA512_WITH_BLOCK_ALT_METHOD_M0",
    "HAVE_BLAKE2",
    "HAVE_HMAC",
    "HAVE_PBKDF2",
    "HAVE_AES",
    "HAVE_MATH",
    "HAVE_RNG",
    "HAVE_RNG_RFC6979",
    "HAVE_RNG_SP800_90A",
    "HAVE_ECC",
    "HAVE_ECC_WEIERSTRASS",
    "HAVE_ECC_TWISTED_EDWARDS",
    "HAVE_ECC_MONTGOMERY",
    "HAVE_SECP256K1_CURVE",
    "HAVE_SECP256R1_CURVE",
    "HAVE_SECP384R1_CURVE",
    "HAVE_SECP521R1_CURVE",
    "HAVE_FR256V1_CURVE",
    "HAVE_STARK256_CURVE",
    "HAVE_BRAINPOOL_P256R1_CURVE",
    "HAVE_BRAINPOOL_P256T1_CURVE",
    "HAVE_BRAINPOOL_P320R1_CURVE",
    "HAVE_BRAINPOOL_P320T1_CURVE",
    "HAVE_BRAINPOOL_P384R1_CURVE",
    "HAVE_BRAINPOOL_P384T1_CURVE",
    "HAVE_BRAINPOOL_P512R1_CURVE",
    "HAVE_BRAINPOOL_P512T1_CURVE",
    "HAVE_BLS12_381_G1_CURVE",
    "HAVE_CV25519_CURVE",
    "HAVE_CV448_CURVE",
    "HAVE_ED25519_CURVE",
    "HAVE_ED448_CURVE",
    "HAVE_ECDH",
    "HAVE_ECDSA",
    "HAVE_EDDSA",
    "HAVE_ECSCHNORR",
    "HAVE_X25519",
    "HAVE_X448",
    "HAVE_AES_GCM",
    "HAVE_CMAC",
    "HAVE_AES_SIV",
    "APP_INSTALL_PARAMS_DATA",
];

const FILTERED_CFLAGS: [&str; 17] = [
    "-c",
    "-Wall",
    "-Wextra",
    "-Wno-main",
    "-Werror=int-to-pointer-cast",
    "-Wno-error=int-conversion",
    "-Wimplicit-fallthrough",
    "-Wvla",
    "-Wundef",
    "-Wshadow",
    "-Wformat=2",
    "-Wformat-security",
    "-Wwrite-strings",
    "-MMD",
    "-MT",
    "-MF",
    "-o",
];

fn main() {
    let args = Args::parse();
    let cur_dir = env::current_dir().expect("Failed to get current directory");

    let path = Path::new(&args.app_path);
    env::set_current_dir(path).expect("Failed to set current directory");

    match args.device.as_str() {
        "nanox" => {
            env::set_var("TARGET", "nanox");
            env::set_var("BOLOS_SDK", env::var("NANOX_SDK").unwrap());
        }
        "nanosplus" => {
            env::set_var("TARGET", "nanos2");
            env::set_var("BOLOS_SDK", env::var("NANOSP_SDK").unwrap());
        }
        "stax" => {
            env::set_var("TARGET", "stax");
            env::set_var("BOLOS_SDK", env::var("STAX_SDK").unwrap());
        }
        "flex" => {
            env::set_var("TARGET", "flex");
            env::set_var("BOLOS_SDK", env::var("FLEX_SDK").unwrap());
        }
        "apex_p" => {
            env::set_var("TARGET", "apex_p");
            env::set_var("BOLOS_SDK", env::var("APEX_P_SDK").unwrap());
        }
        _ => panic!("Unsupported device type. Supported types are: nanox, nanosplus, stax, flex."),
    }

    let output = Command::new("make")
        .args(["--trace", "--dry-run"])
        .output()
        .expect("Failed to execute command");

    let s_out = String::from_utf8_lossy(&output.stdout);

    env::set_current_dir(cur_dir).expect("Failed to reset current directory");

    let mut define_file = File::create(format!("./c_sdk_build_{}.defines", args.device.as_str()))
        .expect("Failed to create file");

    let mut cflags_file = File::create(format!("./c_sdk_build_{}.cflags", args.device.as_str()))
        .expect("Failed to create cflags file");

    for line in s_out.lines() {
        //println!("Processing line: {}", line);
        if line.contains("clang -c") {
            line.split_whitespace().for_each(|word| {
                if word.starts_with("-D"){
                    // Write the word to the file, removing the "-D" prefix
                    let v = word.trim_start_matches("-D").split('=').collect::<Vec<&str>>();
                    //let bool = FILTERED_DEFINES.iter().any(|&x| x == v[0]);
                    //if !bool {
                        write!(define_file, "#define ").unwrap();
                        match v.len() {
                            1 => write!(define_file, "{}", v[0]).unwrap(),
                            2 => write!(define_file, "{} {}", v[0], v[1]).unwrap(),
                            _ => panic!("Unexpected format for define: {}", word),
                        }
                        writeln!(define_file).unwrap();
                    //}
                }
                else if word.starts_with("-I") {}
                else if word.starts_with("-") {
                    //let bool = FILTERED_CFLAGS.iter().any(|&x| x == word);
                    //if !bool {
                        // Write the word to the cflags file
                        writeln!(cflags_file, "{}", word).unwrap();
                    //}
                }
            });
            //writeln!(cflags_file, "-Wno-unused-command-line-argument").unwrap();
            break;
        }
    }

    // Compare output files with reference files
    let ref_define_file = format!("references/c_sdk_build_{}.defines", args.device.as_str());
    let ref_cflags_file = format!("references/c_sdk_build_{}.cflags", args.device.as_str());
    let curr_define_file = format!("c_sdk_build_{}.defines", args.device.as_str());
    let curr_cflags_file = format!("c_sdk_build_{}.cflags", args.device.as_str());

    let curr_define_contents = std::fs::read_to_string(&curr_define_file).expect("Failed to read current defines file");
    let curr_cflags_contents = std::fs::read_to_string(&curr_cflags_file).expect("Failed to read current cflags file");
    let ref_define_contents = std::fs::read_to_string(&ref_define_file).expect("Failed to read reference defines file");
    let ref_cflags_contents = std::fs::read_to_string(&ref_cflags_file).expect("Failed to read reference cflags file");

    if curr_define_contents != ref_define_contents {
        eprintln!("Current defines file does not match reference for target {}", args.device.as_str());
    }

    if curr_cflags_contents != ref_cflags_contents {
        eprintln!("Current cflags file does not match reference for target {}", args.device.as_str());
    }

}
