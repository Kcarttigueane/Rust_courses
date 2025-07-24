use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Local, Utc};

// Structure pour le gestionnaire de fichiers
#[derive(Debug)]
struct FileManager {
    current_directory: String,
    operations_count: u32,
    session_start: DateTime<Local>,
    operation_log: Vec<String>,
}

// Énumération pour les différentes opérations
#[derive(Debug)]
enum FileOperation {
    Read,
    Write,
    Modify,
    Delete,
    List,
    ChangeDirectory,
    CreateDirectory,
    ShowLog,
    ShowStats,
    Exit,
}

// Énumération pour les résultats d'opération
#[derive(Debug)]
enum OperationResult {
    Success(String),
    Error(String),
}

// Implémentation des méthodes pour FileManager
impl FileManager {
    // Constructeur
    fn new() -> Self {
        FileManager {
            current_directory: String::from("."),
            operations_count: 0,
            session_start: Local::now(),
            operation_log: Vec::new(),
        }
    }

    // Méthode utilitaire pour logger les opérations avec timestamps
    fn log_operation(&mut self, operation: &str, details: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!("[{}] {}: {}", timestamp, operation, details);
        self.operation_log.push(log_entry);
        self.operations_count += 1;
    }

    // Méthode pour formater une date système
    fn format_system_time(time: SystemTime) -> String {
        match time.duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let datetime: DateTime<Utc> = DateTime::from_timestamp(duration.as_secs() as i64, 0)
                    .unwrap_or_else(|| Utc::now());
                datetime.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string()
            },
            Err(_) => "Date inconnue".to_string(),
        }
    }

    // Méthode pour lire un fichier (démontre ownership)
    fn read_file(&mut self, filename: &str) -> OperationResult {
        match fs::read_to_string(filename) {
            Ok(content) => {
                self.log_operation("LECTURE", filename);
                OperationResult::Success(content)
            },
            Err(e) => {
                self.log_operation("ERREUR_LECTURE", &format!("{}: {}", filename, e));
                OperationResult::Error(format!("Erreur lors de la lecture: {}", e))
            },
        }
    }

    // Méthode pour écrire dans un fichier
    fn write_file(&mut self, filename: &str, content: &str) -> OperationResult {
        match fs::write(filename, content) {
            Ok(_) => {
                self.log_operation("ECRITURE", filename);
                OperationResult::Success(format!("Fichier '{}' écrit avec succès", filename))
            },
            Err(e) => {
                self.log_operation("ERREUR_ECRITURE", &format!("{}: {}", filename, e));
                OperationResult::Error(format!("Erreur lors de l'écriture: {}", e))
            },
        }
    }

    // Méthode pour modifier un fichier (ajouter du contenu)
    fn modify_file(&mut self, filename: &str, additional_content: &str) -> OperationResult {
        // Lire le contenu existant
        let existing_content = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                self.log_operation("ERREUR_MODIFICATION", &format!("{}: {}", filename, e));
                return OperationResult::Error(format!("Erreur lors de la lecture: {}", e));
            }
        };

        // Combiner avec le nouveau contenu
        let new_content = format!("{}\n{}", existing_content, additional_content);

        match fs::write(filename, new_content) {
            Ok(_) => {
                self.log_operation("MODIFICATION", filename);
                OperationResult::Success(format!("Fichier '{}' modifié avec succès", filename))
            },
            Err(e) => {
                self.log_operation("ERREUR_MODIFICATION", &format!("{}: {}", filename, e));
                OperationResult::Error(format!("Erreur lors de la modification: {}", e))
            },
        }
    }

    // Méthode pour supprimer définitivement un fichier
    fn delete_file(&mut self, filename: &str) -> OperationResult {
        if !Path::new(filename).exists() {
            self.log_operation("ERREUR_SUPPRESSION", &format!("{}: fichier introuvable", filename));
            return OperationResult::Error(format!("Le fichier '{}' n'existe pas", filename));
        }

        match fs::remove_file(filename) {
            Ok(_) => {
                self.log_operation("SUPPRESSION", filename);
                OperationResult::Success(format!("Fichier '{}' supprimé définitivement", filename))
            },
            Err(e) => {
                self.log_operation("ERREUR_SUPPRESSION", &format!("{}: {}", filename, e));
                OperationResult::Error(format!("Erreur lors de la suppression: {}", e))
            },
        }
    }

    // Méthode pour lister les fichiers du répertoire courant
    fn list_files(&mut self) -> OperationResult {
        // Clone current_directory to avoid borrow conflicts
        let current_dir = self.current_directory.clone();

        match fs::read_dir(&current_dir) {
            Ok(entries) => {
                let mut files = Vec::new();

                // Utilisation d'une boucle for pour parcourir les entrées
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            if let Some(name) = path.file_name() {
                                if let Some(name_str) = name.to_str() {
                                    let file_type = if path.is_dir() { "[DIR] " } else { "[FILE]" };

                                    // Obtenir les métadonnées pour la date et la taille
                                    match path.metadata() {
                                        Ok(metadata) => {
                                            let modified_time = metadata.modified()
                                                .map(|time| Self::format_system_time(time))
                                                .unwrap_or_else(|_| "Date inconnue".to_string());

                                            let size = if path.is_file() {
                                                format!("{} octets", metadata.len())
                                            } else {
                                                "-".to_string()
                                            };

                                            files.push(format!("{} {:30} | {:20} | {}",
                                                               file_type, name_str, modified_time, size));
                                        },
                                        Err(_) => {
                                            files.push(format!("{} {}", file_type, name_str));
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => continue,
                    }
                }

                // Now we can safely log since we're using the cloned string
                self.log_operation("LISTAGE", &current_dir);

                let header = format!("Type  | {:30} | {:20} | Taille", "Nom", "Date de modification");
                let separator = "-".repeat(80);
                let result = format!("{}\n{}\n{}", header, separator, files.join("\n"));

                OperationResult::Success(result)
            }
            Err(e) => {
                // Use the cloned string here too
                self.log_operation("ERREUR_LISTAGE", &format!("{}: {}", current_dir, e));
                OperationResult::Error(format!("Erreur lors du listage: {}", e))
            }
        }
    }

    // Méthode pour changer de répertoire
    fn change_directory(&mut self, path: &str) -> OperationResult {
        if Path::new(path).is_dir() {
            let old_dir = self.current_directory.clone();
            self.current_directory = path.to_string();
            self.log_operation("CHANGEMENT_REP", &format!("{} -> {}", old_dir, path));
            OperationResult::Success(format!("Répertoire changé vers: {}", path))
        } else {
            self.log_operation("ERREUR_CHANGEMENT_REP", &format!("{}: répertoire introuvable", path));
            OperationResult::Error(format!("Le répertoire '{}' n'existe pas", path))
        }
    }

    // Méthode pour créer un répertoire
    fn create_directory(&mut self, path: &str) -> OperationResult {
        match fs::create_dir_all(path) {
            Ok(_) => {
                self.log_operation("CREATION_REP", path);
                OperationResult::Success(format!("Répertoire '{}' créé avec succès", path))
            },
            Err(e) => {
                self.log_operation("ERREUR_CREATION_REP", &format!("{}: {}", path, e));
                OperationResult::Error(format!("Erreur lors de la création du répertoire: {}", e))
            },
        }
    }

    // Méthode pour afficher les statistiques
    fn show_stats(&self) {
        let current_time = Local::now();
        let session_duration = current_time.signed_duration_since(self.session_start);

        println!("=== Statistiques ===");
        println!("Session démarrée le: {}", self.session_start.format("%Y-%m-%d à %H:%M:%S"));
        println!("Heure actuelle: {}", current_time.format("%Y-%m-%d à %H:%M:%S"));
        println!("Durée de la session: {} minutes", session_duration.num_minutes());
        println!("Répertoire courant: {}", self.current_directory);
        println!("Nombre d'opérations effectuées: {}", self.operations_count);
        println!("==================");
    }

    // Méthode pour afficher le journal des opérations
    fn show_operation_log(&self, limit: Option<usize>) {
        println!("=== Journal des Opérations ===");

        let logs_to_show = match limit {
            Some(n) => {
                let start = if self.operation_log.len() > n {
                    self.operation_log.len() - n
                } else {
                    0
                };
                &self.operation_log[start..]
            },
            None => &self.operation_log[..]
        };

        if logs_to_show.is_empty() {
            println!("Aucune opération enregistrée");
        } else {
            for log_entry in logs_to_show {
                println!("{}", log_entry);
            }
        }
        println!("=============================");
    }
}

// Fonction pour obtenir l'entrée utilisateur
fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Erreur lors de la lecture");
    input.trim().to_string()
}

// Fonction pour parser l'opération utilisateur
fn parse_operation(input: &str) -> Option<FileOperation> {
    match input.to_lowercase().as_str() {
        "1" | "lire" | "read" => Some(FileOperation::Read),
        "2" | "ecrire" | "write" => Some(FileOperation::Write),
        "3" | "modifier" | "modify" => Some(FileOperation::Modify),
        "4" | "supprimer" | "delete" => Some(FileOperation::Delete),
        "5" | "lister" | "list" => Some(FileOperation::List),
        "6" | "cd" | "changer" => Some(FileOperation::ChangeDirectory),
        "7" | "mkdir" | "creer" => Some(FileOperation::CreateDirectory),
        "8" | "journal" | "log" => Some(FileOperation::ShowLog),
        "9" | "stats" | "statistiques" => Some(FileOperation::ShowStats),
        "10" | "quitter" | "exit" => Some(FileOperation::Exit),
        _ => None,
    }
}

// Fonction principale
fn main() {
    println!("=== Gestionnaire de Fichiers en Rust ===");
    println!("Utilise les concepts: Ownership, Loops, Match, Impl");
    println!("========================================");

    let mut file_manager = FileManager::new();

    // Boucle principale (utilisation de loop)
    loop {
        println!("\n--- Menu Principal ---");
        println!("1. Lire un fichier");
        println!("2. Écrire dans un fichier");
        println!("3. Modifier un fichier");
        println!("4. Supprimer un fichier");
        println!("5. Lister les fichiers");
        println!("6. Changer de répertoire");
        println!("7. Créer un répertoire");
        println!("8. Afficher le journal");
        println!("9. Afficher les statistiques");
        println!("10. Quitter");

        let choice = get_user_input("\nChoisissez une option (1-10): ");

        // Utilisation de match pour traiter les choix
        match parse_operation(&choice) {
            Some(operation) => {
                // Utilisation d'un autre match pour traiter chaque opération
                match operation {
                    FileOperation::Read => {
                        let filename = get_user_input("Nom du fichier à lire: ");
                        let result = file_manager.read_file(&filename);

                        match result {
                            OperationResult::Success(content) => {
                                println!("=== Contenu du fichier '{}' ===", filename);
                                println!("{}", content);
                                println!("=== Fin du contenu ===");
                            }
                            OperationResult::Error(e) => println!("❌ {}", e),
                        }
                    }

                    FileOperation::Write => {
                        let filename = get_user_input("Nom du fichier: ");
                        let content = get_user_input("Contenu à écrire: ");
                        let result = file_manager.write_file(&filename, &content);

                        match result {
                            OperationResult::Success(msg) => println!("✅ {}", msg),
                            OperationResult::Error(e) => println!("❌ {}", e),
                        }
                    }

                    FileOperation::Modify => {
                        let filename = get_user_input("Nom du fichier à modifier: ");
                        let additional_content = get_user_input("Contenu à ajouter: ");
                        let result = file_manager.modify_file(&filename, &additional_content);

                        match result {
                            OperationResult::Success(msg) => println!("✅ {}", msg),
                            OperationResult::Error(e) => println!("❌ {}", e),
                        }
                    }

                    FileOperation::Delete => {
                        let filename = get_user_input("Nom du fichier à supprimer: ");
                        println!("⚠️  ATTENTION: Cette action est irréversible!");
                        let confirmation = get_user_input("Confirmez-vous? (oui/non): ");

                        // Utilisation d'une boucle while pour la confirmation
                        let mut confirmed = false;
                        let conf_lower = confirmation.to_lowercase();

                        if conf_lower == "oui" || conf_lower == "o" || conf_lower == "yes" || conf_lower == "y" {
                            confirmed = true;
                        }

                        if confirmed {
                            let result = file_manager.delete_file(&filename);
                            match result {
                                OperationResult::Success(msg) => println!("✅ {}", msg),
                                OperationResult::Error(e) => println!("❌ {}", e),
                            }
                        } else {
                            println!("🚫 Suppression annulée");
                        }
                    }

                    FileOperation::List => {
                        let result = file_manager.list_files();
                        match result {
                            OperationResult::Success(files) => {
                                println!("=== Fichiers dans '{}' ===", file_manager.current_directory);
                                println!("{}", files);
                                println!("=== Fin de la liste ===");
                            }
                            OperationResult::Error(e) => println!("❌ {}", e),
                        }
                    }

                    FileOperation::ChangeDirectory => {
                        let path = get_user_input("Nouveau répertoire: ");
                        let result = file_manager.change_directory(&path);
                        match result {
                            OperationResult::Success(msg) => println!("✅ {}", msg),
                            OperationResult::Error(e) => println!("❌ {}", e),
                        }
                    }

                    FileOperation::CreateDirectory => {
                        let path = get_user_input("Nom du répertoire à créer: ");
                        let result = file_manager.create_directory(&path);
                        match result {
                            OperationResult::Success(msg) => println!("✅ {}", msg),
                            OperationResult::Error(e) => println!("❌ {}", e),
                        }
                    }

                    FileOperation::ShowLog => {
                        let limit_input = get_user_input("Nombre d'entrées à afficher (laissez vide pour tout): ");
                        let limit = if limit_input.trim().is_empty() {
                            None
                        } else {
                            limit_input.trim().parse::<usize>().ok()
                        };
                        file_manager.show_operation_log(limit);
                    }

                    FileOperation::ShowStats => {
                        file_manager.show_stats();
                    }

                    FileOperation::Exit => {
                        println!("👋 Merci d'avoir utilisé le gestionnaire de fichiers!");
                        file_manager.show_stats();
                        break; // Sort de la boucle principale
                    }
                }
            }
            None => {
                println!("❌ Option invalide. Veuillez choisir entre 1 et 10.");

                // Démonstration d'une boucle while avec compteur
                let mut attempts = 3;
                while attempts > 0 {
                    println!("Il vous reste {} tentative(s)", attempts);
                    attempts -= 1;

                    if attempts == 0 {
                        println!("💡 Astuce: Utilisez les numéros 1-10 pour naviguer dans le menu");
                        break;
                    }
                }
            }
        }
    }
}