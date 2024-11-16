use std::{thread, time};

fn main() {
    println!("Starting OLAP server...");

    // 模拟服务的主循环
    let interval = time::Duration::from_secs(5); // 每隔5秒执行一次任务

    loop {
        if let Err(e) = perform_server_task() {
            eprintln!("Error: {}", e);
            break; // 如果发生严重错误，退出程序
        }

        println!("OLAP server is running...");
        thread::sleep(interval); // 每5秒执行一次任务
    }

    println!("OLAP server is shutting down.");
}

fn perform_server_task() -> Result<(), String> {
    // 模拟任务，直接返回成功
    println!("Performing server task...");
    Ok(()) // 始终返回 Ok，表示任务总是成功
}
