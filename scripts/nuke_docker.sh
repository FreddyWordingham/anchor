#!/bin/bash

set -euo pipefail

# Stop all running containers
docker stop $(docker ps -aq)

# Remove all containers
docker rm $(docker ps -aq)

# Remove all images
docker rmi $(docker images -aq)

# Remove all volumes
docker system prune -a

# Remove all networks
docker network prune -f

# Remove all builds
docker builder prune -a
