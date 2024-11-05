mod logger;

fn main() {
    let log_path = "/var/log/syslog";         // Caminho do log de entrada
    let output_path = "filtered_logs.txt";    // Caminho do arquivo de saída

    // Imprime a mensagem antes de iniciar o monitoramento
    println!("Monitorando logs...");

    match logger::monitor_logs(log_path, output_path) {
        Ok(_) => {}, // Não precisa de ação aqui, pois já estamos monitorando
        Err(e) => eprintln!("Erro ao monitorar logs: {}", e),
    }
}


