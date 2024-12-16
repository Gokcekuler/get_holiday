# Get Holidays

CLI app to list the next 5 holidays using https://date.nager.at - for supported countries. In this application, if the number of holidays remaining until the end of the year is less than 5, the remaining number of holidays is completed from the next year.

## Requirements

Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

```

## Usage

After downloading the code to your local first you can run command ``cargo build`` and after you can run command
``cargo run -- "country cod"``. For example, you can run it by giving the command ``cargo run -- DE`` for Germany and ``cargo run -- FR`` for France.

If you ran the project in one of the main or NextHolidays branches, it is recommended that you delete the ``holidays_cache.json`` file before running it in the other branch. Because these two branches are actually designed to show how it will work when two different logics are applied in the project.

