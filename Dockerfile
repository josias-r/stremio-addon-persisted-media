FROM node:23-alpine

WORKDIR /app

# Enable corepack and install pnpm
RUN corepack enable && corepack prepare pnpm@latest --activate

# Copy package files and install dependencies
COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

# Copy application source code
COPY . .

# Environment variables
ENV PORT=7000
ENV DB_PATH=/app/data/torrents.db

# Expose port
EXPOSE 7000

# Start server
CMD ["pnpm", "start"]
