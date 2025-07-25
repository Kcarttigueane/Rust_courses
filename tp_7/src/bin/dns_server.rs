use tokio::net::UdpSocket;
use std::sync::Arc;
use std::net::SocketAddr;
use colored::*;
use clap::Parser;

use dns_client_server::{
    DnsMessage, DnsRecordType, DnsRecord, SimpleDnsDatabase
};

#[derive(Parser)]
#[command(name = "dns-server")]
#[command(about = "Un serveur DNS simple en Rust")]
struct Args {
    /// Port d'écoute du serveur DNS
    #[arg(short, long, default_value = "5353")]
    port: u16,

    /// Adresse d'écoute
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,

    /// Mode verbeux
    #[arg(short, long)]
    verbose: bool,
}

struct DnsServer {
    socket: Arc<UdpSocket>,
    database: SimpleDnsDatabase,
    verbose: bool,
}

impl DnsServer {
    async fn new(addr: &str, verbose: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(addr).await?;
        println!("🚀 {} Serveur DNS démarré sur {}", "INFO".green().bold(), addr.cyan());

        let database = SimpleDnsDatabase::new();

        // Afficher les enregistrements disponibles
        println!("📚 {} Enregistrements DNS chargés:", "DATABASE".blue().bold());
        for (name, ip) in database.list_records() {
            println!("   {} -> {}", name.yellow(), ip.to_string().green());
        }
        println!();

        Ok(DnsServer {
            socket: Arc::new(socket),
            database,
            verbose,
        })
    }

    async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("👂 {} En attente de requêtes DNS...\n", "LISTENING".green().bold());

        let mut buffer = vec![0u8; 512]; // Buffer standard pour DNS

        loop {
            match self.socket.recv_from(&mut buffer).await {
                Ok((size, client_addr)) => {
                    if self.verbose {
                        println!("📨 {} Requête reçue de {} ({} bytes)",
                                 "REQUEST".cyan().bold(),
                                 client_addr.to_string().yellow(),
                                 size.to_string().magenta()
                        );
                    }

                    // Traiter la requête dans une tâche séparée
                    let socket_clone = self.socket.clone();
                    let data = buffer[..size].to_vec();
                    let database = self.database.clone();
                    let verbose = self.verbose;

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_query(
                            socket_clone,
                            client_addr,
                            data,
                            database,
                            verbose
                        ).await {
                            eprintln!("❌ {} Erreur traitement requête: {}",
                                      "ERROR".red().bold(), e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("❌ {} Erreur réception: {}", "ERROR".red().bold(), e);
                }
            }
        }
    }

    async fn handle_query(
        socket: Arc<UdpSocket>,
        client_addr: SocketAddr,
        data: Vec<u8>,
        database: SimpleDnsDatabase,
        verbose: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {

        // Parser la requête DNS
        let query = match DnsMessage::from_bytes(&data) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("❌ {} Impossible de parser la requête: {}",
                          "PARSE_ERROR".red().bold(), e);
                return Ok(());
            }
        };

        if verbose {
            println!("🔍 {} Requête parsée:", "PARSE".blue().bold());
            println!("   ID: {}", query.header.id.to_string().cyan());
            println!("   Questions: {}", query.questions.len().to_string().magenta());

            for (i, question) in query.questions.iter().enumerate() {
                println!("   Question {}: {} (Type: {:?})",
                         i + 1,
                         question.name.yellow(),
                         question.qtype
                );
            }
        }

        // Créer la réponse
        let mut response = DnsMessage::new_response(&query);

        // Traiter chaque question
        for question in &query.questions {
            match question.qtype {
                DnsRecordType::A => {
                    if let Some(ip) = database.lookup(&question.name) {
                        let record = DnsRecord::new_a_record(
                            question.name.clone(),
                            ip,
                            300  // TTL de 5 minutes
                        );
                        response.answers.push(record);
                        response.header.ancount += 1;

                        println!("✅ {} Résolu: {} -> {}",
                                 "RESOLVED".green().bold(),
                                 question.name.yellow(),
                                 ip.to_string().green()
                        );
                    } else {
                        // Domain non trouvé
                        response.header.rcode = 3; // NXDOMAIN
                        println!("❌ {} Domaine non trouvé: {}",
                                 "NXDOMAIN".red().bold(),
                                 question.name.yellow()
                        );
                    }
                }
                _ => {
                    // Type de requête non supporté
                    response.header.rcode = 4; // NOTIMP
                    println!("❌ {} Type de requête non supporté: {:?}",
                             "UNSUPPORTED".red().bold(),
                             question.qtype
                    );
                }
            }
        }

        // Envoyer la réponse
        let response_bytes = response.to_bytes();

        match socket.send_to(&response_bytes, client_addr).await {
            Ok(sent) => {
                if verbose {
                    println!("📤 {} Réponse envoyée à {} ({} bytes)",
                             "RESPONSE".green().bold(),
                             client_addr.to_string().yellow(),
                             sent.to_string().magenta()
                    );
                    println!("   Réponses: {}", response.header.ancount.to_string().cyan());
                    println!("   Code: {}",
                             match response.header.rcode {
                                 0 => "NOERROR".green(),
                                 3 => "NXDOMAIN".red(),
                                 4 => "NOTIMP".yellow(),
                                 _ => "UNKNOWN".red(),
                             }
                    );
                    println!(); // Ligne vide pour la lisibilité
                }
            }
            Err(e) => {
                eprintln!("❌ {} Erreur envoi réponse: {}", "SEND_ERROR".red().bold(), e);
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("{}", "🌐 SERVEUR DNS SIMPLE EN RUST 🦀".blue().bold());
    println!("{}", "=".repeat(40).blue());
    println!("📋 Configuration:");
    println!("   • Adresse: {}", format!("{}:{}", args.address, args.port).cyan());
    println!("   • Mode verbeux: {}", if args.verbose { "ON".green() } else { "OFF".red() });
    println!("   • Protocole: {}", "UDP".yellow());
    println!();

    let addr = format!("{}:{}", args.address, args.port);
    let server = DnsServer::new(&addr, args.verbose).await?;

    println!("💡 {} Pour tester le serveur:", "ASTUCE".yellow().bold());
    println!("   cargo run --bin dns_client -- google.com");
    println!("   nslookup google.com 127.0.0.1 -port={}", args.port);
    println!("   dig @127.0.0.1 -p {} google.com", args.port);
    println!();

    // Capturer Ctrl+C pour un arrêt propre
    tokio::select! {
        result = server.start() => {
            if let Err(e) = result {
                eprintln!("❌ {} Erreur serveur: {}", "FATAL".red().bold(), e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("\n🛑 {} Arrêt du serveur...", "SHUTDOWN".yellow().bold());
        }
    }

    println!("👋 {} Serveur arrêté", "BYE".green().bold());
    Ok(())
}