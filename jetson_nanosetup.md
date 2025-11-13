# Copy binary from main laptop
scp 100.106.28.84:/usr/local/bin/chat-app /tmp/
sudo cp /tmp/chat-app /usr/local/bin/
sudo chmod +x /usr/local/bin/chat-app

# Test
chat-app client --address 100.106.28.84:8080
