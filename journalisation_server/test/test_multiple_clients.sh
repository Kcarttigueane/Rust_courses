#!/bin/bash

# Script pour tester le serveur avec plusieurs clients simultanés
# Usage: ./test_multiple_clients.sh

echo "🧪 === TEST MULTI-CLIENTS POUR LE SERVEUR DE JOURNALISATION ==="
echo "Ce script va simuler 3 clients connectés simultanément"
echo "Assurez-vous que le serveur est démarré avec: cargo run"
echo ""

# Fonction pour simuler un client
simulate_client() {
    local client_id=$1
    local duration=$2

    echo "🚀 Démarrage du client $client_id"

    # Créer un fichier temporaire avec les commandes
    cat << EOF > /tmp/client_${client_id}_commands.txt
Hello from client $client_id!
ping
This is message 2 from client $client_id
stats
Testing concurrent logging - client $client_id
help
Final message from client $client_id before disconnect
quit
EOF

    # Se connecter et envoyer les commandes
    (
        sleep 1
        while IFS= read -r line; do
            echo "$line"
            sleep $duration
        done < /tmp/client_${client_id}_commands.txt
    ) | nc 127.0.0.1 8080 &

    # Nettoyer le fichier temporaire
    sleep $((duration * 8 + 5))
    rm -f /tmp/client_${client_id}_commands.txt
}

# Vérifier si netcat est disponible
if ! command -v nc &> /dev/null; then
    echo "❌ netcat (nc) n'est pas installé"
    echo "💡 Sur Ubuntu/Debian: sudo apt install netcat"
    echo "💡 Sur macOS: brew install netcat"
    exit 1
fi

# Vérifier si le serveur répond
echo "🔍 Vérification de la connexion au serveur..."
if ! timeout 3 bash -c "echo 'ping' | nc 127.0.0.1 8080 > /dev/null 2>&1"; then
    echo "❌ Le serveur ne répond pas sur 127.0.0.1:8080"
    echo "💡 Démarrez d'abord le serveur avec: cargo run"
    exit 1
fi

echo "✅ Serveur détecté!"
echo ""

echo "🎯 Lancement de 3 clients simultanés..."
echo "   • Client A: Messages toutes les 2 secondes"
echo "   • Client B: Messages toutes les 3 secondes"
echo "   • Client C: Messages toutes les 1 seconde"
echo ""

# Lancer les clients en parallèle
simulate_client "A" 2 &
CLIENT_A_PID=$!

sleep 0.5
simulate_client "B" 3 &
CLIENT_B_PID=$!

sleep 0.5
simulate_client "C" 1 &
CLIENT_C_PID=$!

echo "🔄 Clients lancés! PIDs: $CLIENT_A_PID, $CLIENT_B_PID, $CLIENT_C_PID"
echo "⏱️  Les clients vont s'exécuter pendant environ 30 secondes..."
echo ""
echo "👀 Surveillez:"
echo "   • La console du serveur pour voir les connexions"
echo "   • Le fichier logs/server.log pour les messages horodatés"
echo ""

# Attendre que tous les clients finissent
wait $CLIENT_A_PID
wait $CLIENT_B_PID
wait $CLIENT_C_PID

echo "✅ Test terminé!"
echo ""
echo "📊 Résultats à vérifier:"
echo "   • Console du serveur: connexions/déconnexions des 3 clients"
echo "   • Fichier logs/server.log: tous les messages avec horodatage"
echo "   • Ordre des messages: devrait refléter l'exécution concurrente"
echo ""
echo "💡 Astuce: Regardez les timestamps dans les logs pour voir la concurrence!"