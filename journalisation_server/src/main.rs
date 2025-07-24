use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::fs::{File, OpenOptions};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::net::SocketAddr;
use chrono::{DateTime, Local};
use uuid::Uuid;
use std::path::Path;

// Structure pour gérer les informations du client
#[derive(Debug, Clone)]
struct ClientInfo {
    id: String,
    address: SocketAddr,
    connected_at: DateTime<Local>,
}

// Structure principale du serveur
struct LoggingServer {
    log_file: Arc<Mutex<File>>,
    active_clients: Arc<Mutex<Vec<ClientInfo>>>,
}

impl LoggingServer {
    async fn new() -> tokio::io::Result<Self> {
        // Créer le dossier logs s'il n'existe pas
        if !Path::new("logs").exists() {
            tokio::fs::create_dir("logs").await?;
            println!("📁 Dossier 'logs' créé");
        }

        // Ouvrir/créer le fichier de log
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("logs/server.log")
            .await?;

        let server = LoggingServer {
            log_file: Arc::new(Mutex::new(log_file)),
            active_clients: Arc::new(Mutex::new(Vec::new())),
        };

        server.log_server_message("🚀 Serveur de journalisation démarré").await?;

        Ok(server)
    }

    // Méthode pour logger un message du serveur
    async fn log_server_message(&self, message: &str) -> tokio::io::Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_entry = format!("[{}] [SERVER] {}\n", timestamp, message);

        let mut file = self.log_file.lock().await;
        file.write_all(log_entry.as_bytes()).await?;
        file.flush().await?;

        // Afficher aussi dans la console
        print!("{}", log_entry);

        Ok(())
    }

    // Méthode pour logger un message client
    async fn log_client_message(&self, client_info: &ClientInfo, message: &str) -> tokio::io::Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_entry = format!("[{}] [CLIENT:{}] [{}] {}\n",
                                timestamp, client_info.id, client_info.address, message);

        let mut file = self.log_file.lock().await;
        file.write_all(log_entry.as_bytes()).await?;
        file.flush().await?;

        // Afficher aussi dans la console avec couleur
        print!("💬 {}", log_entry);

        Ok(())
    }

    // Ajouter un client à la liste des clients actifs
    async fn add_client(&self, client_info: ClientInfo) {
        let mut clients = self.active_clients.lock().await;
        clients.push(client_info.clone());

        if let Err(e) = self.log_server_message(&format!(
            "✅ Nouveau client connecté: {} [{}] - Total clients: {}",
            client_info.id, client_info.address, clients.len()
        )).await {
            eprintln!("❌ Erreur lors du logging: {}", e);
        }
    }

    // Retirer un client de la liste des clients actifs
    async fn remove_client(&self, client_id: &str) {
        let mut clients = self.active_clients.lock().await;
        clients.retain(|client| client.id != client_id);

        if let Err(e) = self.log_server_message(&format!(
            "❌ Client déconnecté: {} - Clients restants: {}",
            client_id, clients.len()
        )).await {
            eprintln!("❌ Erreur lors du logging: {}", e);
        }
    }

    // Afficher les statistiques du serveur
    async fn show_stats(&self) {
        let clients = self.active_clients.lock().await;
        if let Err(e) = self.log_server_message(&format!(
            "📊 Statistiques - Clients actifs: {}", clients.len()
        )).await {
            eprintln!("❌ Erreur lors du logging: {}", e);
        }

        for client in clients.iter() {
            let duration = Local::now().signed_duration_since(client.connected_at);
            if let Err(e) = self.log_server_message(&format!(
                "   └─ {} [{}] - Connecté depuis {} minutes",
                client.id, client.address, duration.num_minutes()
            )).await {
                eprintln!("❌ Erreur lors du logging: {}", e);
            }
        }
    }

    // Méthode principale pour gérer un client
    async fn handle_client(
        server: Arc<LoggingServer>,
        mut stream: TcpStream,
        client_addr: SocketAddr,
    ) -> tokio::io::Result<()> {
        // Créer les informations du client
        let client_info = ClientInfo {
            id: Uuid::new_v4().to_string()[..8].to_string(), // ID court
            address: client_addr,
            connected_at: Local::now(),
        };

        // Ajouter le client à la liste
        server.add_client(client_info.clone()).await;

        // Envoyer un message de bienvenue au client
        let welcome_msg = format!("🎉 Bienvenue sur le serveur de journalisation!\n");
        let welcome_msg2 = format!("📝 Votre ID: {} | Votre IP: {}\n", client_info.id, client_info.address);
        let welcome_msg3 = format!("💡 Tapez vos messages (ils seront loggés avec horodatage)\n");
        let welcome_msg4 = format!("🔚 Tapez 'quit' pour vous déconnecter\n\n");

        if let Err(e) = stream.write_all(welcome_msg.as_bytes()).await {
            eprintln!("❌ Erreur envoi message: {}", e);
        }
        if let Err(e) = stream.write_all(welcome_msg2.as_bytes()).await {
            eprintln!("❌ Erreur envoi message: {}", e);
        }
        if let Err(e) = stream.write_all(welcome_msg3.as_bytes()).await {
            eprintln!("❌ Erreur envoi message: {}", e);
        }
        if let Err(e) = stream.write_all(welcome_msg4.as_bytes()).await {
            eprintln!("❌ Erreur envoi message: {}", e);
        }

        // Séparer le stream en parties lecture et écriture
        let (read_half, mut write_half) = stream.split();
        let reader = BufReader::new(read_half);
        let mut lines = reader.lines();

        // Boucle principale pour lire les messages du client
        while let Ok(Some(line)) = lines.next_line().await {
            let message = line.trim();

            // Vérifier si le client veut se déconnecter
            if message.to_lowercase() == "quit" || message.to_lowercase() == "exit" {
                let goodbye_msg = format!("👋 Au revoir {}! Déconnexion...\n", client_info.id);
                let _ = write_half.write_all(goodbye_msg.as_bytes()).await;

                if let Err(e) = server.log_client_message(&client_info, "DÉCONNEXION VOLONTAIRE").await {
                    eprintln!("❌ Erreur lors du logging: {}", e);
                }
                break;
            }

            // Ignorer les messages vides
            if message.is_empty() {
                continue;
            }

            // Logger le message du client
            if let Err(e) = server.log_client_message(&client_info, message).await {
                eprintln!("❌ Erreur lors du logging: {}", e);
                continue;
            }

            // Envoyer une confirmation au client
            let confirmation = format!("✅ Message reçu et loggé: '{}'\n", message);
            if let Err(e) = write_half.write_all(confirmation.as_bytes()).await {
                eprintln!("❌ Erreur envoi confirmation: {}", e);
                break;
            }

            // Commandes spéciales
            match message.to_lowercase().as_str() {
                "stats" => {
                    server.show_stats().await;
                    let stats_msg = "📊 Statistiques affichées dans les logs du serveur\n";
                    let _ = write_half.write_all(stats_msg.as_bytes()).await;
                }
                "ping" => {
                    let pong_msg = "🏓 Pong! Serveur actif\n";
                    let _ = write_half.write_all(pong_msg.as_bytes()).await;
                }
                "help" => {
                    let help_msg = "🆘 Commandes disponibles:\n";
                    let help_msg2 = "   - stats: Afficher les statistiques\n";
                    let help_msg3 = "   - ping: Tester la connexion\n";
                    let help_msg4 = "   - help: Afficher cette aide\n";
                    let help_msg5 = "   - quit/exit: Se déconnecter\n\n";
                    let _ = write_half.write_all(help_msg.as_bytes()).await;
                    let _ = write_half.write_all(help_msg2.as_bytes()).await;
                    let _ = write_half.write_all(help_msg3.as_bytes()).await;
                    let _ = write_half.write_all(help_msg4.as_bytes()).await;
                    let _ = write_half.write_all(help_msg5.as_bytes()).await;
                }
                _ => {} // Message normal, déjà traité
            }
        }

        // Nettoyer lors de la déconnexion
        server.remove_client(&client_info.id).await;

        Ok(())
    }

    async fn start(&self, addr: &str) -> tokio::io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        self.log_server_message(&format!("🎯 Serveur en écoute sur {}", addr)).await?;

        // Créer un Arc pour partager le serveur entre les tâches
        let server = Arc::new(LoggingServer {
            log_file: self.log_file.clone(),
            active_clients: self.active_clients.clone(),
        });

        // Tâche pour afficher les statistiques périodiquement
        let stats_server = server.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                stats_server.show_stats().await;
            }
        });

        // Boucle principale d'acceptation des connexions
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let server_clone = server.clone();

                    // Spawner une nouvelle tâche pour chaque client
                    tokio::spawn(async move {
                        if let Err(e) = LoggingServer::handle_client(server_clone, stream, addr).await {
                            eprintln!("❌ Erreur avec le client {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("❌ Erreur d'acceptation de connexion: {}", e);
                    self.log_server_message(&format!("❌ Erreur connexion: {}", e)).await?;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    println!("🌟 === SERVEUR DE JOURNALISATION ASYNCHRONE ===");
    println!("📋 Fonctionnalités:");
    println!("   • Support multi-clients simultanés");
    println!("   • Journalisation avec horodatage précis");
    println!("   • Commandes intégrées (stats, ping, help)");
    println!("   • Gestion propre des déconnexions");
    println!("   • Logs sauvegardés dans logs/server.log");
    println!("{}", "=" .repeat(50));

    // Créer le serveur
    let server = LoggingServer::new().await?;

    // Adresse d'écoute
    let addr = "127.0.0.1:8080";

    println!("🚀 Démarrage du serveur...");
    println!("💡 Pour tester, utilisez: telnet 127.0.0.1 8080");
    println!("💡 Ou: nc 127.0.0.1 8080");
    println!("💡 Ou utilisez le client de test ci-dessous");
    println!("{}", "=" .repeat(50));

    server.start(addr).await
}