use std::io;

struct BankAccount {
    account_number: String,
    holder_name: String,
    balance: f64,
}

impl BankAccount {
    fn new(account_number: String, holder_name: String, starting_balance: f64) -> Self {
        BankAccount {
            account_number,
            holder_name,
            balance: starting_balance,
        }
    }

    fn withdraw_money(&mut self, amount: f64) -> bool {
        if amount <= 0.0 {
            println!("❌ Le montant doit être positif!");
            return false;
        }
        
        if amount > self.balance {
            println!("❌ Pas assez d'argent! Solde actuel: {:.2} €", self.balance);
            return false;
        }
        
        self.balance = self.balance - amount;
        println!("✅ Retrait de {:.2} € effectué!", amount);
        println!("💰 Nouveau solde: {:.2} €", self.balance);
        return true;
    }

    fn show_balance(&self) {
        println!("\n==== INFORMATIONS DU COMPTE ====");
        println!("📋 Numéro: {}", self.account_number);
        println!("👤 Titulaire: {}", self.holder_name);
        println!("💰 Solde: {:.2} €", self.balance);
        println!("================================");
    }

    fn deposit_money(&mut self, amount: f64) -> bool {
        if amount <= 0.0 {
            println!("❌ Le montant doit être positif!");
            return false;
        }
        
        self.balance = self.balance + amount;
        println!("✅ Dépôt de {:.2} € effectué!", amount);
        println!("💰 Nouveau solde: {:.2} €", self.balance);
        return true;
    }

    fn renommer(&self, new_name: String) -> BankAccount {
        BankAccount {
            account_number: self.account_number.clone(),
            holder_name: new_name,
            balance: self.balance,
        }
    }
}

fn show_menu() {
    println!("\n🏦 === MENU BANCAIRE === 🏦");
    println!("1. 💰 Afficher solde");
    println!("2. 💸 Retrait");
    println!("3. 💵 Dépôt");
    println!("4. 📋 Liste comptes");
    println!("5. ✏️  Renommer compte");
    println!("6. 🚪 Quitter");
    println!("========================");
    print!("👉 Votre choix (1-6): ");
}

fn get_user_input() -> String {
    let mut input = String::new();
    
    io::stdin()
        .read_line(&mut input)
        .expect("❌ Erreur lors de la lecture!");
    
    input.trim().to_string()
}

fn get_number_from_user(message: &str) -> f64 {
    loop {
        println!("{}", message);
        let input = get_user_input();
        
        match input.parse::<f64>() {
            Ok(number) => return number,
            Err(_) => println!("❌ Ce n'est pas un nombre valide. Essayez encore."),
        }
    }
}

fn show_all_accounts(accounts: &Vec<BankAccount>) {
    println!("\n📋 === LISTE DES COMPTES ===");
    
    if accounts.is_empty() {
        println!("Aucun compte disponible.");
        return;
    }
    
    for (index, account) in accounts.iter().enumerate() {
        println!("{}. {} - {} - {:.2} €", 
            index, 
            account.account_number, 
            account.holder_name, 
            account.balance
        );
    }
    println!("===========================");
}

fn choose_account(accounts: &Vec<BankAccount>) -> Option<usize> {
    if accounts.is_empty() {
        println!("❌ Aucun compte disponible!");
        return None;
    }
    
    show_all_accounts(accounts);
    
    loop {
        let choice = get_number_from_user("👉 Choisissez un compte (tapez le numéro):") as usize;
        
        if choice < accounts.len() {
            return Some(choice);
        } else {
            println!("❌ Numéro invalide. Choisissez entre 0 et {}.", accounts.len() - 1);
        }
    }
}

fn main() {
    let mut bank_accounts: Vec<BankAccount> = Vec::new();
    
    bank_accounts.push(BankAccount::new(
        "123456".to_string(),
        "Jean Dupont".to_string(),
        1000.0
    ));
    
    bank_accounts.push(BankAccount::new(
        "789012".to_string(),
        "Marie Martin".to_string(),
        2500.0
    ));
    
    bank_accounts.push(BankAccount::new(
        "345678".to_string(),
        "Pierre Durand".to_string(),
        750.0
    ));
    
    let mut current_account_index: Option<usize> = Some(0);
    
    println!("🏦 Bienvenue dans votre système bancaire! 🏦");
    
    loop {
        show_menu();
        let choice = get_user_input();
        
        match choice.as_str() {
            "1" => {
                match current_account_index {
                    Some(index) => bank_accounts[index].show_balance(),
                    None => println!("❌ Aucun compte sélectionné! Choisissez d'abord un compte."),
                }
            },
            
            "2" => {
                match current_account_index {
                    Some(index) => {
                        let amount = get_number_from_user("💸 Combien voulez-vous retirer? (en €):");
                        bank_accounts[index].withdraw_money(amount);
                    },
                    None => println!("❌ Aucun compte sélectionné! Choisissez d'abord un compte."),
                }
            },

            "3" => {
                match current_account_index {
                    Some(index) => {
                        let amount = get_number_from_user("💵 Combien voulez-vous déposer? (en €):");
                        bank_accounts[index].deposit_money(amount);
                    },
                    None => println!("❌ Aucun compte sélectionné! Choisissez d'abord un compte."),
                }
            },
            
            "4" => {
                match choose_account(&bank_accounts) {
                    Some(index) => {
                        current_account_index = Some(index);
                        println!("✅ Compte sélectionné: {}", bank_accounts[index].account_number);
                    },
                    None => println!("❌ Aucun compte sélectionné."),
                }
            },

            "5" => {
                match current_account_index {
                    Some(index) => {
                        println!("👤 Nom actuel: {}", bank_accounts[index].holder_name);
                        println!("✏️  Nouveau nom:");
                        let new_name = get_user_input();
                        if !new_name.is_empty() {
                            let new_account = bank_accounts[index].renommer(new_name);
                            bank_accounts[index] = new_account;
                            println!("✅ Nom changé avec succès!");
                        } else {
                            println!("❌ Le nom ne peut pas être vide.");
                        }
                    },
                    None => println!("❌ Aucun compte sélectionné! Choisissez d'abord un compte."),
                }
            },
            
            "6" => {
                println!("👋 Au revoir et merci d'avoir utilisé notre système bancaire!");
                break;
            },
            
            _ => {
                println!("❌ Choix invalide. Tapez 1, 2, 3, 4, 5 ou 6.");
            }
        }
        
        println!("\n⏸️  Appuyez sur Entrée pour continuer...");
        get_user_input();
    }
}