# Get Holidays

CLI app to list the next 5 holidays using https://date.nager.at - for supported countries. In this application, if the number of holidays remaining until the end of the year is less than 5, it lists the number of holidays left.

## Requirements

Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

```

## Usage

After downloading the code to your local first you can run command ``cargo build`` and after you can run command
``cargo run -- "country cod"``. For example, you can run it by giving the command ``cargo run -- DE`` for Germany and ``cargo run -- FR`` for France.
