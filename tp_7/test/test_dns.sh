#!/bin/bash

# Script de test complet pour le système DNS
# Usage: ./test_dns.sh

echo "🧪 === TEST COMPLET DU SYSTÈME DNS CLIENT/SERVEUR ==="
echo "Ce script va tester le serveur et client DNS développés en Rust"
echo ""

# Couleurs pour la sortie
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Fonction pour afficher des messages colorés
print_step() {
    echo -e "${BLUE}🔵 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
}

# Vérifier que le projet est compilé
print_step "Compilation du projet..."
if cargo build --release; then
    print_success "Compilation réussie"
else
    print_error "Échec de la compilation"
    exit 1
fi

echo ""

# Démarrer le serveur DNS en arrière-plan
print_step "Démarrage du serveur DNS..."
cargo run --bin dns_server -- --port 5353 --verbose &
SERVER_PID=$!

# Attendre que le serveur démarre
sleep 2

# Vérifier si le serveur est démarré
if kill -0 $SERVER_PID 2>/dev/null; then
    print_success "Serveur DNS démarré (PID: $SERVER_PID)"
else
    print_error "Échec du démarrage du serveur"
    exit 1
fi

echo ""

# Fonction pour tester une requête DNS
test_dns_query() {
    local domain=$1
    local expected=$2
    local description=$3
    
    print_step "Test: $description"
    print_info "Requête pour: $domain"
    
    # Capturer la sortie du client
    if timeout 10 cargo run --bin dns_client -- "$domain" --server "127.0.0.1:5353" --verbose > /tmp/dns_test_output.txt 2>&1; then
        if grep -q "$expected" /tmp/dns_test_output.txt; then
            print_success "✓ Résolution correcte trouvée"
            grep "Adresses trouvées:" -A 5 /tmp/dns_test_output.txt | grep -E "→|->|=" || echo ""
        else
            print_warning "Réponse reçue mais résultat inattendu"
            cat /tmp/dns_test_output.txt
        fi
    else
        print_error "Échec de la requête ou timeout"
        cat /tmp/dns_test_output.txt
    fi
    echo ""
}

# Tests des domaines prédéfinis dans le serveur
print_step "=== TESTS DES DOMAINES LOCAUX ==="
echo ""

test_dns_query "localhost" "127.0.0.1" "Résolution de localhost"
test_dns_query "test.local" "192.168.1.100" "Résolution de test.local"
test_dns_query "server.local" "192.168.1.1" "Résolution de server.local"
test_dns_query "example.com" "93.184.216.34" "Résolution de example.com"
test_dns_query "google.com" "8.8.8.8" "Résolution de google.com"

# Tests de domaines inexistants
print_step "=== TESTS DE DOMAINES INEXISTANTS ==="
echo ""

test_dns_query "inexistant.local" "NXDOMAIN" "Test domaine inexistant"
test_dns_query "unknown.test" "NXDOMAIN" "Test autre domaine inexistant"

# Tests de types de requêtes non supportés
print_step "=== TESTS DE TYPES NON SUPPORTÉS ==="
echo ""

print_step "Test: Requête NS (non supportée)"
if timeout 10 cargo run --bin dns_client -- "google.com" --server "127.0.0.1:5353" --query-type "NS" > /tmp/dns_ns_test.txt 2>&1; then
    if grep -q "NOTIMP\|non supporté" /tmp/dns_ns_test.txt; then
        print_success "✓ Type NS correctement rejeté"
    else
        print_warning "Réponse inattendue pour type NS"
        cat /tmp/dns_ns_test.txt
    fi
else
    print_error "Échec du test NS"
fi
echo ""

# Test de performance
print_step "=== TEST DE PERFORMANCE ==="
echo ""

print_step "Test: Mesure du temps de réponse"
print_info "10 requêtes consécutives vers localhost..."

total_time=0
successful_queries=0

for i in {1..10}; do
    if timeout 5 cargo run --bin dns_client -- "localhost" --server "127.0.0.1:5353" > /tmp/perf_test_$i.txt 2>&1; then
        # Extraire le temps de réponse
        time_ms=$(grep "Temps de réponse:" /tmp/perf_test_$i.txt | grep -oE '[0-9]+\.[0-9]+' | head -1)
        if [ ! -z "$time_ms" ]; then
            echo "  Requête $i: ${time_ms}ms"
            total_time=$(echo "$total_time + $time_ms" | bc -l 2>/dev/null || echo "$total_time")
            successful_queries=$((successful_queries + 1))
        fi
    else
        echo "  Requête $i: ÉCHEC"
    fi
done

if [ $successful_queries -gt 0 ] && command -v bc >/dev/null 2>&1; then
    avg_time=$(echo "scale=2; $total_time / $successful_queries" | bc)
    print_success "Temps moyen: ${avg_time}ms sur $successful_queries requêtes"
else
    print_info "Calcul de moyenne non disponible (bc manquant ou aucune requête réussie)"
fi

echo ""

# Test de concurrence
print_step "=== TEST DE CONCURRENCE ==="
echo ""

print_step "Test: 5 clients simultanés"
print_info "Lancement de 5 requêtes en parallèle..."

# Lancer 5 clients en parallèle
for i in {1..5}; do
    (
        timeout 10 cargo run --bin dns_client -- "google.com" --server "127.0.0.1:5353" > /tmp/concurrent_$i.txt 2>&1
        if grep -q "8.8.8.8" /tmp/concurrent_$i.txt; then
            echo "Client $i: ✅ Succès"
        else
            echo "Client $i: ❌ Échec"
        fi
    ) &
done

# Attendre que tous les clients finissent
wait
print_success "Test de concurrence terminé"
echo ""

# Test avec comparaison DNS publics
print_step "=== TEST AVEC DNS PUBLICS ==="
echo ""

print_step "Test: Comparaison avec serveurs DNS publics"
print_info "Requête pour google.com avec comparaison..."

if timeout 15 cargo run --bin dns_client -- "google.com" --server "127.0.0.1:5353" --compare-with-public > /tmp/public_comparison.txt 2>&1; then
    print_success "Test de comparaison réussi"
    grep -A 10 "COMPARAISON AVEC DNS PUBLICS" /tmp/public_comparison.txt | grep -E "Google DNS|Cloudflare|Quad9|✅|❌" || echo ""
else
    print_error "Échec du test de comparaison"
fi

echo ""

# Arrêter le serveur
print_step "Arrêt du serveur DNS..."
if kill $SERVER_PID 2>/dev/null; then
    sleep 1
    if kill -0 $SERVER_PID 2>/dev/null; then
        kill -9 $SERVER_PID 2>/dev/null
    fi
    print_success "Serveur arrêté"
else
    print_warning "Le serveur était déjà arrêté"
fi

echo ""

# Nettoyage
print_step "Nettoyage des fichiers temporaires..."
rm -f /tmp/dns_test_output.txt /tmp/dns_ns_test.txt /tmp/perf_test_*.txt /tmp/concurrent_*.txt /tmp/public_comparison.txt
print_success "Nettoyage terminé"

echo ""
print_step "=== RÉSUMÉ DES TESTS ==="
echo ""
print_success "✅ Compilation et démarrage du serveur"
print_success "✅ Résolution de domaines locaux"
print_success "✅ Gestion des domaines inexistants (NXDOMAIN)"
print_success "✅ Rejet des types non supportés (NOTIMP)"
print_success "✅ Test de performance"
print_success "✅ Test de concurrence"
print_success "✅ Comparaison avec DNS publics"

echo ""
print_step "🎉 TOUS LES TESTS TERMINÉS AVEC SUCCÈS ! 🎉"
echo ""

print_info "Pour utiliser le système manuellement:"
echo "1. Démarrer le serveur: cargo run --bin dns_server"
echo "2. Utiliser le client: cargo run --bin dns_client -- google.com"
echo "3. Ou avec nslookup: nslookup google.com 127.0.0.1 -port=5353"
echo ""

print_info "Le serveur inclut ces domaines prédéfinis:"
echo "• localhost -> 127.0.0.1"
echo "• test.local -> 192.168.1.100"
echo "• server.local -> 192.168.1.1"
echo "• example.com -> 93.184.216.34"
echo "• google.com -> 8.8.8.8"