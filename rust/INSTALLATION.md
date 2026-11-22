# Installation

## Using Cargo
```bash
cargo install rsysmon-cli   # cli version
cargo install rsysmond      # daemon
cargo install rsysmon       # gui
```

## download from releases page

### Linux x86_64
```bash
chmod +x sysmon-cli
sudo mv sysmon-cli /usr/local/bin/
# or
mv sysmon-cli "$HOME/.local/bin/"
```

## From source
```bash
git clone https://github.com/MohamedGonem/sysmon
cd sysmon/rust
# use --features "feature_a,feature_b" to specifiy features as well
cargo install --path crates/sysmon-cli
```

> [!IMPORTANT]
> Currently depends on `clap`

