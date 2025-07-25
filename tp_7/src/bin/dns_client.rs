use tokio::net::UdpSocket;
use std::time::{Duration, Instant};
use colored::*;
use clap::Parser;

use dns_client_server::{
    DnsMessage, DnsRecordType, DnsQuestion
};

#[derive(Parser)]
#[command(name = "dns-client")]
#[command(about = "Un client DNS simple en Rust")]
struct Args {
    /// Nom de domaine à résoudre
    domain: String,

    /// Serveur DNS à utiliser
    #[arg(short, long, default_value = "127.0.0.1:5353")]
    server: String,

    /// Type de requête (A, NS, CNAME, etc.)
    #[arg(short, long, default_value = "A")]
    query_type: String,

    /// Timeout en millisecondes
    #[arg(short, long, default_value = "5000")]
    timeout: u64,

    /// Mode verbeux
    #[arg(short, long)]
    verbose: bool,

    /// Serveur DNS public pour comparaison
    #[arg(long)]
    compare_with_public: bool,
}

struct DnsClient {
    socket: UdpSocket,
    verbose: bool,
}

impl DnsClient {
    async fn new(verbose: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        Ok(DnsClient { socket, verbose })
    }

    async fn query(
        &self,
        domain: &str,
        server: &str,
        qtype: DnsRecordType,
        timeout_ms: u64,
    ) -> Result<DnsMessage, Box<dyn std::error::Error>> {

        if self.verbose {
            println!("🔍 {} Création de la requête DNS", "QUERY".blue().bold());
            println!("   Domaine: {}", domain.yellow());
            println!("   Serveur: {}", server.cyan());
            println!("   Type: {:?}", qtype);
        }

        // Créer la requête DNS
        let query = DnsMessage::new_query(domain.to_string(), qtype);
        let query_bytes = query.to_bytes();

        if self.verbose {
            println!("   ID de transaction: {}", query.header.id.to_string().magenta());
            println!("   Taille de la requête: {} bytes", query_bytes.len().to_string().cyan());
        }

        // Mesurer le temps de requête
        let start_time = Instant::now();

        // Envoyer la requête
        if self.verbose {
            println!("📤 {} Envoi de la requête...", "SEND".green().bold());
        }

        self.socket.send_to(&query_bytes, server).await?;

        // Attendre la réponse avec timeout
        let mut buffer = vec![0u8; 512];

        let response_result = tokio::time::timeout(
            Duration::from_millis(timeout_ms),
            self.socket.recv(&mut buffer)
        ).await;

        let response_size = match response_result {
            Ok(Ok(size)) => size,
            Ok(Err(e)) => return Err(format!("Erreur réception: {}", e).into()),
            Err(_) => return Err(format!("Timeout après {}ms", timeout_ms).into()),
        };

        let response_time = start_time.elapsed();

        if self.verbose {
            println!("📨 {} Réponse reçue", "RECEIVE".green().bold());
            println!("   Taille: {} bytes", response_size.to_string().cyan());
            println!("   Temps: {:.2}ms", response_time.as_secs_f64() * 1000.0);
        }

        // Parser la réponse
        let response = DnsMessage::from_bytes(&buffer[..response_size])?;

        if self.verbose {
            println!("🔍 {} Réponse parsée", "PARSE".blue().bold());
            println!("   ID: {}", response.header.id.to_string().magenta());
            println!("   QR: {}", if response.header.qr { "Response".green() } else { "Query".red() });
            println!("   RCODE: {}",
                     match response.header.rcode {
                         0 => "NOERROR".green(),
                         3 => "NXDOMAIN".red(),
                         4 => "NOTIMP".yellow(),
                         _ => "UNKNOWN".red(),
                     }
            );
            println!("   Réponses: {}", response.header.ancount.to_string().cyan());
        }

        Ok(response)
    }

    fn display_results(&self, domain: &str, response: &DnsMessage, response_time: Duration) {
        println!("\n{}", "📊 RÉSULTATS".blue().bold());
        println!("{}", "=".repeat(40).blue());

        println!("🎯 {} {}", "Domaine:".bold(), domain.yellow());
        println!("⏱️  {} {:.2}ms", "Temps de réponse:".bold(),
                 response_time.as_secs_f64() * 1000.0);

        println!("📝 {} {}", "Code de réponse:".bold(),
                 match response.header.rcode {
                     0 => "✅ NOERROR (Succès)".green(),
                     3 => "❌ NXDOMAIN (Domaine inexistant)".red(),
                     4 => "⚠️  NOTIMP (Non implémenté)".yellow(),
                     _ => "❓ Code inconnu".red(),
                 }
        );

        if !response.answers.is_empty() {
            println!("\n📍 {} Adresses trouvées:", "RÉPONSES".green().bold());
            for (i, answer) in response.answers.iter().enumerate() {
                match answer.get_ip() {
                    Some(ip) => {
                        println!("   {}. {} -> {} (TTL: {}s)",
                                 i + 1,
                                 answer.name.yellow(),
                                 ip.to_string().green(),
                                 answer.ttl.to_string().cyan()
                        );
                    }
                    None => {
                        println!("   {}. {} -> [Données: {} bytes]",
                                 i + 1,
                                 answer.name.yellow(),
                                 answer.data.len().to_string().magenta()
                        );
                    }
                }
            }
        } else {
            println!("\n❌ {} Aucune réponse trouvée", "RÉSULTAT".red().bold());
        }
    }

    async fn compare_with_public_dns(&self, domain: &str, qtype: DnsRecordType) {
        println!("\n{}", "🌐 COMPARAISON AVEC DNS PUBLICS".yellow().bold());
        println!("{}", "=".repeat(40).yellow());

        let public_servers = vec![
            ("Google DNS", "8.8.8.8:53"),
            ("Cloudflare DNS", "1.1.1.1:53"),
            ("Quad9 DNS", "9.9.9.9:53"),
        ];

        for (name, server) in public_servers {
            print!("🔍 Test avec {} ({})... ", name.cyan(), server.yellow());

            match self.query(domain, server, qtype, 3000).await {
                Ok(response) => {
                    if response.header.rcode == 0 && !response.answers.is_empty() {
                        let ip = response.answers[0].get_ip()
                            .map(|ip| ip.to_string())
                            .unwrap_or_else(|| "Données binaires".to_string());
                        println!("✅ {}", ip.green());
                    } else {
                        println!("❌ {}", "Aucune réponse".red());
                    }
                }
                Err(e) => {
                    println!("❌ {}", format!("Erreur: {}", e).red());
                }
            }
        }
    }
}

fn parse_query_type(type_str: &str) -> Result<DnsRecordType, String> {
    match type_str.to_uppercase().as_str() {
        "A" => Ok(DnsRecordType::A),
        "NS" => Ok(DnsRecordType::NS),
        "CNAME" => Ok(DnsRecordType::CNAME),
        "PTR" => Ok(DnsRecordType::PTR),
        "MX" => Ok(DnsRecordType::MX),
        "AAAA" => Ok(DnsRecordType::AAAA),
        _ => Err(format!("Type de requête non supporté: {}", type_str)),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("{}", "🔍 CLIENT DNS SIMPLE EN RUST 🦀".blue().bold());
    println!("{}", "=".repeat(40).blue());

    // Parser le type de requête
    let query_type = parse_query_type(&args.query_type)?;

    if args.verbose {
        println!("📋 {} Configuration:", "CLIENT".blue().bold());
        println!("   • Domaine: {}", args.domain.yellow());
        println!("   • Serveur: {}", args.server.cyan());
        println!("   • Type: {:?}", query_type);
        println!("   • Timeout: {}ms", args.timeout.to_string().magenta());
        println!();
    }

    // Créer le client
    let client = DnsClient::new(args.verbose).await?;

    // Mesurer le temps total
    let total_start = Instant::now();

    // Effectuer la requête
    match client.query(&args.domain, &args.server, query_type, args.timeout).await {
        Ok(response) => {
            let total_time = total_start.elapsed();
            client.display_results(&args.domain, &response, total_time);

            // Comparaison avec DNS publics si demandée
            if args.compare_with_public {
                client.compare_with_public_dns(&args.domain, query_type).await;
            }
        }
        Err(e) => {
            println!("\n❌ {} Erreur lors de la requête:", "ERREUR".red().bold());
            println!("   {}", e.to_string().red());

            println!("\n💡 {} Suggestions:", "DÉPANNAGE".yellow().bold());
            println!("   • Vérifiez que le serveur DNS est démarré");
            println!("   • Vérifiez l'adresse du serveur: {}", args.server.cyan());
            println!("   • Essayez d'augmenter le timeout avec -t");
            println!("   • Utilisez -v pour plus de détails");

            std::process::exit(1);
        }
    }

    Ok(())
}