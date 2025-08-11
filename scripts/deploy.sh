#!/bin/bash

# CIM Domain Workflow Deployment Script
# Copyright 2024 The CIM Consortium. All rights reserved.

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DEFAULT_NAMESPACE="cim-workflow"
DEFAULT_ENVIRONMENT="production"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Help function
show_help() {
    cat << EOF
CIM Domain Workflow Deployment Script

Usage: $0 [OPTIONS] COMMAND

Commands:
    init        Initialize deployment environment
    build       Build Docker images
    deploy      Deploy to Kubernetes
    upgrade     Upgrade existing deployment
    rollback    Rollback to previous version
    status      Check deployment status
    logs        View deployment logs
    cleanup     Clean up resources

Options:
    -e, --environment ENV    Target environment (dev|staging|production) [default: production]
    -n, --namespace NS       Kubernetes namespace [default: cim-workflow]
    -v, --version VERSION    Version tag for deployment [default: latest]
    -f, --force             Force deployment without confirmation
    -h, --help              Show this help message

Examples:
    $0 init --environment staging
    $0 build --version v1.2.3
    $0 deploy --environment production --namespace cim-prod
    $0 upgrade --version v1.2.4 --force
    $0 rollback --version v1.2.2

EOF
}

# Parse command line arguments
parse_args() {
    ENVIRONMENT="${DEFAULT_ENVIRONMENT}"
    NAMESPACE="${DEFAULT_NAMESPACE}"
    VERSION="latest"
    FORCE=false
    COMMAND=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            -e|--environment)
                ENVIRONMENT="$2"
                shift 2
                ;;
            -n|--namespace)
                NAMESPACE="$2"
                shift 2
                ;;
            -v|--version)
                VERSION="$2"
                shift 2
                ;;
            -f|--force)
                FORCE=true
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            init|build|deploy|upgrade|rollback|status|logs|cleanup)
                COMMAND="$1"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    if [[ -z "$COMMAND" ]]; then
        log_error "Command is required"
        show_help
        exit 1
    fi
}

# Validate environment
validate_environment() {
    case $ENVIRONMENT in
        dev|staging|production)
            ;;
        *)
            log_error "Invalid environment: $ENVIRONMENT. Must be one of: dev, staging, production"
            exit 1
            ;;
    esac
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local missing_tools=()
    
    # Check required tools
    if ! command -v docker &> /dev/null; then
        missing_tools+=("docker")
    fi
    
    if ! command -v kubectl &> /dev/null; then
        missing_tools+=("kubectl")
    fi
    
    if ! command -v helm &> /dev/null; then
        missing_tools+=("helm")
    fi
    
    if ! command -v cargo &> /dev/null; then
        missing_tools+=("cargo")
    fi
    
    if [[ ${#missing_tools[@]} -gt 0 ]]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi
    
    # Check kubectl connection
    if ! kubectl cluster-info &> /dev/null; then
        log_error "Cannot connect to Kubernetes cluster"
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Initialize deployment environment
init_deployment() {
    log_info "Initializing deployment environment for $ENVIRONMENT..."
    
    # Create namespace if it doesn't exist
    if ! kubectl get namespace "$NAMESPACE" &> /dev/null; then
        log_info "Creating namespace: $NAMESPACE"
        kubectl create namespace "$NAMESPACE"
    fi
    
    # Create deployment directories
    mkdir -p "$PROJECT_ROOT/deployment/k8s/$ENVIRONMENT"
    mkdir -p "$PROJECT_ROOT/deployment/helm/environments"
    mkdir -p "$PROJECT_ROOT/deployment/configs/$ENVIRONMENT"
    
    # Generate environment-specific configurations
    generate_config_files
    
    # Setup secrets
    setup_secrets
    
    log_success "Deployment environment initialized"
}

# Generate configuration files
generate_config_files() {
    log_info "Generating configuration files for $ENVIRONMENT..."
    
    # Generate Helm values file
    cat > "$PROJECT_ROOT/deployment/helm/environments/$ENVIRONMENT-values.yaml" << EOF
# CIM Workflow Helm values for $ENVIRONMENT
replicaCount: $(get_replica_count)
image:
  repository: cim-workflow
  tag: "$VERSION"
  pullPolicy: IfNotPresent
service:
  type: ClusterIP
  port: 8080
ingress:
  enabled: true
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-$(get_cert_issuer)
  hosts:
    - host: $(get_hostname)
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: cim-workflow-tls-$ENVIRONMENT
      hosts:
        - $(get_hostname)
resources:
  limits:
    cpu: $(get_cpu_limit)
    memory: $(get_memory_limit)
  requests:
    cpu: $(get_cpu_request)
    memory: $(get_memory_request)
autoscaling:
  enabled: $(get_autoscaling_enabled)
  minReplicas: $(get_min_replicas)
  maxReplicas: $(get_max_replicas)
  targetCPUUtilizationPercentage: 80
postgresql:
  enabled: $(get_postgresql_enabled)
  global:
    postgresql:
      auth:
        database: "cim_workflows_$ENVIRONMENT"
nats:
  enabled: $(get_nats_enabled)
  cluster:
    enabled: true
    replicas: $(get_nats_replicas)
monitoring:
  enabled: $(get_monitoring_enabled)
  serviceMonitor:
    enabled: true
    namespace: monitoring
EOF

    # Generate application config
    cat > "$PROJECT_ROOT/deployment/configs/$ENVIRONMENT/app-config.toml" << EOF
[server]
host = "0.0.0.0"
port = 8080
environment = "$ENVIRONMENT"

[database]
max_connections = $(get_db_max_connections)
connection_timeout_seconds = 30

[nats]
cluster_url = "nats://nats-cluster:4222"
max_reconnects = 10
reconnect_delay_ms = 1000

[logging]
level = "$(get_log_level)"
format = "json"

[metrics]
enabled = true
prometheus_endpoint = "/metrics"

[tracing]
enabled = $(get_tracing_enabled)
jaeger_endpoint = "http://jaeger-collector:14268"
EOF

    log_success "Configuration files generated"
}

# Environment-specific configuration functions
get_replica_count() {
    case $ENVIRONMENT in
        dev) echo "1" ;;
        staging) echo "2" ;;
        production) echo "3" ;;
    esac
}

get_hostname() {
    case $ENVIRONMENT in
        dev) echo "workflow-dev.company.com" ;;
        staging) echo "workflow-staging.company.com" ;;
        production) echo "workflow.company.com" ;;
    esac
}

get_cert_issuer() {
    case $ENVIRONMENT in
        dev) echo "staging" ;;
        *) echo "prod" ;;
    esac
}

get_cpu_limit() {
    case $ENVIRONMENT in
        dev) echo "500m" ;;
        staging) echo "1000m" ;;
        production) echo "2000m" ;;
    esac
}

get_memory_limit() {
    case $ENVIRONMENT in
        dev) echo "512Mi" ;;
        staging) echo "1Gi" ;;
        production) echo "2Gi" ;;
    esac
}

get_cpu_request() {
    case $ENVIRONMENT in
        dev) echo "250m" ;;
        staging) echo "500m" ;;
        production) echo "1000m" ;;
    esac
}

get_memory_request() {
    case $ENVIRONMENT in
        dev) echo "256Mi" ;;
        staging) echo "512Mi" ;;
        production) echo "1Gi" ;;
    esac
}

get_autoscaling_enabled() {
    case $ENVIRONMENT in
        dev) echo "false" ;;
        *) echo "true" ;;
    esac
}

get_min_replicas() {
    case $ENVIRONMENT in
        dev) echo "1" ;;
        staging) echo "2" ;;
        production) echo "3" ;;
    esac
}

get_max_replicas() {
    case $ENVIRONMENT in
        dev) echo "2" ;;
        staging) echo "5" ;;
        production) echo "10" ;;
    esac
}

get_postgresql_enabled() {
    case $ENVIRONMENT in
        dev) echo "true" ;;
        *) echo "false" ;;  # Use external DB in staging/prod
    esac
}

get_nats_enabled() {
    case $ENVIRONMENT in
        dev) echo "true" ;;
        *) echo "false" ;;  # Use external NATS in staging/prod
    esac
}

get_nats_replicas() {
    case $ENVIRONMENT in
        dev) echo "1" ;;
        staging) echo "3" ;;
        production) echo "3" ;;
    esac
}

get_monitoring_enabled() {
    case $ENVIRONMENT in
        dev) echo "false" ;;
        *) echo "true" ;;
    esac
}

get_db_max_connections() {
    case $ENVIRONMENT in
        dev) echo "10" ;;
        staging) echo "20" ;;
        production) echo "50" ;;
    esac
}

get_log_level() {
    case $ENVIRONMENT in
        dev) echo "debug" ;;
        staging) echo "info" ;;
        production) echo "warn" ;;
    esac
}

get_tracing_enabled() {
    case $ENVIRONMENT in
        dev) echo "true" ;;
        staging) echo "true" ;;
        production) echo "false" ;;  # Enable only if needed in prod
    esac
}

# Setup secrets
setup_secrets() {
    log_info "Setting up secrets for $ENVIRONMENT..."
    
    # Database secret
    if [[ $ENVIRONMENT != "dev" ]]; then
        kubectl create secret generic cim-db-secret \
            --namespace="$NAMESPACE" \
            --from-literal=url="postgresql://username:password@db-host:5432/cim_workflows_$ENVIRONMENT" \
            --dry-run=client -o yaml | kubectl apply -f -
    fi
    
    # NATS secret (if using external NATS)
    if [[ $ENVIRONMENT != "dev" ]]; then
        kubectl create secret generic cim-nats-secret \
            --namespace="$NAMESPACE" \
            --from-literal=url="nats://nats-cluster-$ENVIRONMENT:4222" \
            --dry-run=client -o yaml | kubectl apply -f -
    fi
    
    log_success "Secrets configured"
}

# Build Docker images
build_images() {
    log_info "Building Docker images with tag: $VERSION"
    
    cd "$PROJECT_ROOT"
    
    # Build the main application image
    log_info "Building main application image..."
    docker build -t "cim-workflow:$VERSION" \
        -f docker/Dockerfile \
        --build-arg VERSION="$VERSION" \
        --build-arg BUILD_DATE="$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
        --build-arg VCS_REF="$(git rev-parse HEAD)" \
        .
    
    # Build migration tools image (if needed)
    if [[ -f "migration/tools/Dockerfile" ]]; then
        log_info "Building migration tools image..."
        docker build -t "cim-migration-tools:$VERSION" \
            -f migration/tools/Dockerfile \
            migration/tools/
    fi
    
    log_success "Docker images built successfully"
}

# Deploy to Kubernetes
deploy_to_k8s() {
    log_info "Deploying to Kubernetes environment: $ENVIRONMENT"
    
    # Confirmation check (unless forced)
    if [[ "$FORCE" != "true" ]]; then
        echo -n "Deploy to $ENVIRONMENT environment in namespace $NAMESPACE? (y/N): "
        read -r confirmation
        if [[ "$confirmation" != "y" && "$confirmation" != "Y" ]]; then
            log_warn "Deployment cancelled by user"
            exit 0
        fi
    fi
    
    # Use Helm for deployment
    helm upgrade --install cim-workflow \
        "$PROJECT_ROOT/deployment/helm/cim-workflow" \
        --namespace="$NAMESPACE" \
        --values="$PROJECT_ROOT/deployment/helm/environments/$ENVIRONMENT-values.yaml" \
        --set image.tag="$VERSION" \
        --wait \
        --timeout=10m
    
    # Wait for rollout to complete
    kubectl rollout status deployment/cim-workflow-engine -n "$NAMESPACE"
    
    log_success "Deployment completed successfully"
}

# Upgrade existing deployment
upgrade_deployment() {
    log_info "Upgrading deployment to version: $VERSION"
    
    # Check if deployment exists
    if ! kubectl get deployment cim-workflow-engine -n "$NAMESPACE" &> /dev/null; then
        log_error "Deployment does not exist. Run 'deploy' command first."
        exit 1
    fi
    
    # Perform upgrade
    deploy_to_k8s
    
    log_success "Upgrade completed successfully"
}

# Rollback deployment
rollback_deployment() {
    log_info "Rolling back to version: $VERSION"
    
    # Confirmation check (unless forced)
    if [[ "$FORCE" != "true" ]]; then
        echo -n "Rollback to version $VERSION in $ENVIRONMENT? (y/N): "
        read -r confirmation
        if [[ "$confirmation" != "y" && "$confirmation" != "Y" ]]; then
            log_warn "Rollback cancelled by user"
            exit 0
        fi
    fi
    
    # Use Helm rollback if version not specified, otherwise redeploy with specific version
    if [[ "$VERSION" == "latest" ]]; then
        helm rollback cim-workflow -n "$NAMESPACE"
    else
        deploy_to_k8s
    fi
    
    log_success "Rollback completed successfully"
}

# Check deployment status
check_status() {
    log_info "Checking deployment status for environment: $ENVIRONMENT"
    
    echo "=== Namespace: $NAMESPACE ==="
    kubectl get all -n "$NAMESPACE"
    
    echo -e "\n=== Ingress Status ==="
    kubectl get ingress -n "$NAMESPACE"
    
    echo -e "\n=== Pod Status ==="
    kubectl get pods -n "$NAMESPACE" -o wide
    
    echo -e "\n=== Service Status ==="
    kubectl get services -n "$NAMESPACE"
    
    # Check application health
    echo -e "\n=== Application Health ==="
    if kubectl get service cim-workflow-engine-service -n "$NAMESPACE" &> /dev/null; then
        kubectl port-forward -n "$NAMESPACE" service/cim-workflow-engine-service 8080:8080 &
        PORT_FORWARD_PID=$!
        sleep 2
        
        if curl -s http://localhost:8080/health &> /dev/null; then
            log_success "Application is healthy"
        else
            log_error "Application health check failed"
        fi
        
        kill $PORT_FORWARD_PID 2> /dev/null || true
    fi
}

# View deployment logs
view_logs() {
    log_info "Viewing logs for environment: $ENVIRONMENT"
    
    # Get the most recent pod
    POD_NAME=$(kubectl get pods -n "$NAMESPACE" -l app=cim-workflow-engine \
               -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    
    if [[ -n "$POD_NAME" ]]; then
        log_info "Showing logs for pod: $POD_NAME"
        kubectl logs -n "$NAMESPACE" "$POD_NAME" -f
    else
        log_error "No pods found for cim-workflow-engine"
        exit 1
    fi
}

# Cleanup resources
cleanup_resources() {
    log_info "Cleaning up resources for environment: $ENVIRONMENT"
    
    # Confirmation check (unless forced)
    if [[ "$FORCE" != "true" ]]; then
        echo -n "Delete all resources in namespace $NAMESPACE? This cannot be undone! (y/N): "
        read -r confirmation
        if [[ "$confirmation" != "y" && "$confirmation" != "Y" ]]; then
            log_warn "Cleanup cancelled by user"
            exit 0
        fi
    fi
    
    # Uninstall Helm release
    if helm list -n "$NAMESPACE" | grep -q cim-workflow; then
        helm uninstall cim-workflow -n "$NAMESPACE"
    fi
    
    # Delete namespace (if not default)
    if [[ "$NAMESPACE" != "default" ]]; then
        kubectl delete namespace "$NAMESPACE" --ignore-not-found
    fi
    
    log_success "Cleanup completed"
}

# Main execution
main() {
    parse_args "$@"
    validate_environment
    check_prerequisites
    
    case $COMMAND in
        init)
            init_deployment
            ;;
        build)
            build_images
            ;;
        deploy)
            build_images
            deploy_to_k8s
            ;;
        upgrade)
            build_images
            upgrade_deployment
            ;;
        rollback)
            rollback_deployment
            ;;
        status)
            check_status
            ;;
        logs)
            view_logs
            ;;
        cleanup)
            cleanup_resources
            ;;
        *)
            log_error "Unknown command: $COMMAND"
            show_help
            exit 1
            ;;
    esac
    
    log_success "Operation completed successfully!"
}

# Run main function with all arguments
main "$@"