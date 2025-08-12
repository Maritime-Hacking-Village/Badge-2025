use crate::{
    apps::rhai_repl::{ReplFlowControl, ReplInputSender, ReplOutputReceiver, ReplPrintControl},
    platform::usb_cdc_io::UsbCdcIo,
};
use alloc::string::String;
use defmt::{error, format, info, warn};
use embassy_futures::{select, select::Either};
use embassy_rp::{pac::sio::regs::DoorbellOutSet, peripherals::USB, usb::Driver};
use embassy_usb::class::cdc_acm::CdcAcmClass;
use embedded_io_async::{Read, Write};
use utf8_parser::Utf8Parser;

const SPLASH_0: &str = "╱╭━━━╮╱╭━━╮╱╭━━━╮╱╭━━━╮╱╭━━━╮╱╭━━━╮╱╭━━━╮╱╭━╮╱╭╮╱╭━━━━╮╱╭━━╮╱╭━━━╮╱╭╮╱╱╱
╱╰╮╭╮┃╱╰┫┣╯╱┃╭━━╯╱┃╭━━╯╱┃╭━━╯╱┃╭━╮┃╱┃╭━━╯╱┃┃╰╮┃┃╱┃╭╮╭╮┃╱╰┫┣╯╱┃╭━╮┃╱┃┃╱╱╱
╱╱┃┃┃┃╱╱┃┃╱╱┃╰━━╮╱┃╰━━╮╱┃╰━━╮╱┃╰━╯┃╱┃╰━━╮╱┃╭╮╰╯┃╱╰╯┃┃╰╯╱╱┃┃╱╱┃┃╱┃┃╱┃┃╱╱╱
╱╱┃┃┃┃╱╱┃┃╱╱┃╭━━╯╱┃╭━━╯╱┃╭━━╯╱┃╭╮╭╯╱┃╭━━╯╱┃┃╰╮┃┃╱╱╱┃┃╱╱╱╱┃┃╱╱┃╰━╯┃╱┃┃╱╭╮
╱╭╯╰╯┃╱╭┫┣╮╱┃┃╱╱╱╱┃┃╱╱╱╱┃╰━━╮╱┃┃┃╰╮╱┃╰━━╮╱┃┃╱┃┃┃╱╱╱┃┃╱╱╱╭┫┣╮╱┃╭━╮┃╱┃╰━╯┃
╱╰━━━╯╱╰━━╯╱╰╯╱╱╱╱╰╯╱╱╱╱╰━━━╯╱╰╯╰━╯╱╰━━━╯╱╰╯╱╰━╯╱╱╱╰╯╱╱╱╰━━╯╱╰╯╱╰╯╱╰━━━╯";
const SPLASH_1: &str = r#"
╱╱╱╱╱╱╱╱╱╭━━━╮╱╭━━━╮╱╭━━━╮╱╭━━━━╮╱╭━━━╮╱╭━━━╮╱╭╮╱╱╭╮╱╭━━━╮╱╭━━━╮╱╱╱╱╱╱╱╱
╱╱╱╱╱╱╱╱╱╰╮╭╮┃╱┃╭━━╯╱┃╭━╮┃╱┃╭╮╭╮┃╱┃╭━╮┃╱┃╭━╮┃╱┃╰╮╭╯┃╱┃╭━━╯╱┃╭━╮┃╱╱╱╱╱╱╱╱
╱╱╱╱╱╱╱╱╱╱┃┃┃┃╱┃╰━━╮╱┃╰━━╮╱╰╯┃┃╰╯╱┃╰━╯┃╱┃┃╱┃┃╱╰╮╰╯╭╯╱┃╰━━╮╱┃╰━╯┃╱╱╱╱╱╱╱╱
╱╱╱╱╱╱╱╱╱╱┃┃┃┃╱┃╭━━╯╱╰━━╮┃╱╱╱┃┃╱╱╱┃╭╮╭╯╱┃┃╱┃┃╱╱╰╮╭╯╱╱┃╭━━╯╱┃╭╮╭╯╱╱╱╱╱╱╱╱
╱╱╱╱╱╱╱╱╱╭╯╰╯┃╱┃╰━━╮╱┃╰━╯┃╱╱╱┃┃╱╱╱┃┃┃╰╮╱┃╰━╯┃╱╱╱┃┃╱╱╱┃╰━━╮╱┃┃┃╰╮╱╱╱╱╱╱╱╱
╱╱╱╱╱╱╱╱╱╰━━━╯╱╰━━━╯╱╰━━━╯╱╱╱╰╯╱╱╱╰╯╰━╯╱╰━━━╯╱╱╱╰╯╱╱╱╰━━━╯╱╰╯╰━╯╱╱╱╱╱╱╱╱

                       .-----------------.
                       |        |\_..--"/ `.,______
                       | __..--``  DC33/     ___(_()
~-,.,-~"^"~-,.,-~"^"~,.| \____________/   ,'`^"~-,.,-~"^"~-,.,-~"^"~-,._
,.,-~"^"~-,.,-~"^"~-,.,'-----------------'-~",.,-~"^"~-,.,-~"^"~-,.,-~"^
"~-,.,-~"^"~-,.,-~"^"~-,.,-~"^"~-,.,-~"^"~-,.,-~"^"~-,.,-~"^"~-,.,-~"^"~

"#;
const SPLASH_2: &str = "
Hint:
Ctrl-C to interrupt execution
Ctrl-D to reset engine

";
const SPLASH_EXIT: &str = r#"
                                                           |
                                                           :
                                                      |---------|
                                                     +-----------+
                                                     | o o o o o |
                                                  /| |___________|
                                                 / | | o o o o o |
                                                /--| |___________|
                                                 / | | o o o o o |
                                              /----| |___________|
[][][] [][][] [][][] [][][] [][][] [][][] [][][] | | | o o o o o |
[][][] [][][] [][][] [][][] [][][] [][][] [][][] | | |___________|      |>
[][][] [][][] [][][] [][][] [][][] [][][] [][][] | |/____________/      |
/========================================================================\
\ MHV - Las Vegas                                            Bon Voyage! /
@\______________________________________________________________________/
/@\/@\~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
"~-,.,-~"^"~-,.,-~"^"~-,.,-~"^"~-,.,-~"^"~-,.,-~"^"~-,.,-~"^"~-,.,-~"^"~
"#;
const PROMPT_0: &str = "repl> ";
const PROMPT_1: &str = "    > ";

fn apply_backspace(input: &str) -> String {
    let mut result = String::new();

    for c in input.chars() {
        if c == '\u{007F}' {
            result.pop();
        } else {
            result.push(c);
        }
    }

    result
}

// TODO: Remove tty control characters.
fn remove_ascii_ctrl(input: &str) -> String {
    input.chars().filter(|c| !c.is_control()).collect()
}

fn contains_ctrl_c(buf: &[u8], count: usize) -> bool {
    assert!(count <= buf.len());

    for i in 0..count {
        if buf[i] == 0x03 {
            return true;
        }
    }

    false
}

fn contains_ctrl_d(buf: &[u8], count: usize) -> bool {
    assert!(count <= buf.len());

    for i in 0..count {
        if buf[i] == 0x04 {
            return true;
        }
    }

    false
}

macro_rules! write_output {
    ($io:ident,$out:expr) => {
        match $out {
            ReplPrintControl::Done(output) => {
                match output.as_str() {
                    _ => {
                        $io.write_all(&format!("=> {}\n", output).as_bytes())
                            .await?;
                    }
                }

                break;
            }
            ReplPrintControl::Continue(output) => {
                for line in output.lines() {
                    $io.write_all(&format!("-> {}\n", line).as_bytes()).await?;
                }
            }
            ReplPrintControl::Debug(output) => {
                for line in output.lines() {
                    $io.write_all(&format!("~> {}\n", line).as_bytes()).await?;
                }
            }
        }
    };
}

pub async fn cli_inner<T: Write + Read>(
    io: &mut T,
    repl_in_tx: ReplInputSender,
    repl_out_rx: ReplOutputReceiver,
) -> Result<(), T::Error> {
    let mut input = String::new();
    let mut utf8_parser = Utf8Parser::new();
    let mut buf = [0u8; 128];
    let sio = embassy_rp::pac::SIO;

    // Wait for first user input (any key press).
    let _ = io.read(&mut buf).await?;
    io.write_all(SPLASH_0.as_bytes()).await?;
    io.write_all(SPLASH_1.as_bytes()).await?;
    io.write_all(SPLASH_2.as_bytes()).await?;

    'session: loop {
        // Print everything that is pending. This shouldn't ever trigger.
        while let Ok(output) = repl_out_rx.try_receive() {
            error!("Flushing extra messages from Rhai engine!");
            write_output!(io, output);
        }

        // Render the prompt
        let prompt = if input.is_empty() {
            PROMPT_0
        } else if input.ends_with("\n") {
            PROMPT_1
        } else {
            ""
        };

        io.write_all(prompt.as_bytes()).await?;
        // Read bytes and pass to the UTF-8 parser
        let n = io.read(&mut buf).await?;
        // error!("BUF {:?}", buf);

        // TODO: These two conditions can be optimized.
        if contains_ctrl_c(&buf, n) {
            warn!("User sent C-c!");
            io.write_all("\n".as_bytes()).await?;
            input.clear();
            continue 'session;
        }

        if contains_ctrl_d(&buf, n) {
            warn!("User sent C-d!");
            io.write_all(SPLASH_EXIT.as_bytes()).await?;
            io.write_all("\nReset engine. Press any key to continue.\n".as_bytes())
                .await?;
            repl_in_tx.send(ReplFlowControl::Break).await;
            break 'session;
        }

        for b in &buf[..n] {
            match utf8_parser.push(*b).unwrap_or(None) {
                Some('\n') | Some('\r') if Some('\\') == input.chars().last() => {
                    // Remove the backslash and add a newline.
                    input.pop();
                    input.push('\n');
                }
                Some('\n') | Some('\r') => {
                    // Attempt to evaluate by sending to the Rhai core.
                    let sanitized_input = remove_ascii_ctrl(&apply_backspace(&input));
                    repl_in_tx
                        .send(ReplFlowControl::Input(sanitized_input))
                        .await;
                    let mut int_buf = [0u8; 32];

                    'eval: loop {
                        match select::select(repl_out_rx.receive(), io.read(&mut int_buf)).await {
                            Either::First(output) => {
                                write_output!(io, output);
                            }
                            Either::Second(Ok(count)) => {
                                // NOTE: Doesn't work if this core is suspended polling a channel.
                                if contains_ctrl_c(&int_buf, count) {
                                    warn!("User sent C-c!");
                                    sio.doorbell_out_set().write_value(DoorbellOutSet(0xAA));
                                }

                                loop {
                                    let output = repl_out_rx.receive().await;
                                    write_output!(io, output);
                                }

                                break 'eval;
                            }
                            Either::Second(Err(err)) => return Err(err),
                        }
                    }

                    input.clear();
                }
                Some(char) => {
                    input.push(char);
                }
                None => {}
            }
        }
    }

    Ok(())
}

#[embassy_executor::task]
pub async fn cli_task(
    mut class: CdcAcmClass<'static, Driver<'static, USB>>,
    repl_in_tx: ReplInputSender,
    repl_out_rx: ReplOutputReceiver,
) -> ! {
    class.wait_connection().await;
    let mut io = UsbCdcIo(class);

    loop {
        info!("Connected");
        cli_inner(&mut io, repl_in_tx.clone(), repl_out_rx.clone())
            .await
            .unwrap_or_else(|err| {
                warn!("Error handling client: {:?}", err);
            });
        info!("Disconnected");
    }
}
