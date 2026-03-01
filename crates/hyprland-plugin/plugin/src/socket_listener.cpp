#include "socket_listener.h"
#include "handlers.h"
#include "globals.h"

#include <thread>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>
#include <vector>
#include <string>
#include <cstring>
#include <iostream>

std::string getSocketPath() {
    std::filesystem::path buf;
    if (const char *runtime_path = getenv("XDG_RUNTIME_DIR"); runtime_path != nullptr) {
        if (const char *instance = getenv("HYPRLAND_INSTANCE_SIGNATURE"); instance != nullptr) {
            buf = std::filesystem::path(runtime_path) / "hypr" / instance;
        } else {
            buf = std::filesystem::path(runtime_path);
        }
    } else {
        if (const char *uid = getenv("UID"); uid != nullptr) {
            buf = std::filesystem::path("/run/user/") / uid;
        } else {
            buf = std::filesystem::path("/tmp");
        }
    }
    buf /= "hyprshell.sock";
    return buf.string();
}

void listenOnSocket(const std::atomic<bool> &shouldStop, const int sock) {
    std::vector<char> buffer;
    char readBuf[1024];
    while (!shouldStop.load()) {
        // check shouldStop at least every second
        fd_set rfds;
        FD_ZERO(&rfds);
        FD_SET(sock, &rfds);
        // 500 ms
        timeval tv{0, 500000};
        if (const int ret = select(sock + 1, &rfds, nullptr, nullptr, &tv); ret <= 0)
            continue;

        const ssize_t bytesRead = read(sock, readBuf, sizeof(readBuf));
        if (bytesRead <= 0)
            break;

        for (ssize_t i = 0; i < bytesRead; ++i) {
            if (readBuf[i] == '\0') {
                if (!buffer.empty()) {
                    try {
                        std::string msgStr(buffer.begin(), buffer.end());
                        auto jsonMsg = nlohmann::json::parse(msgStr);
                        handleMessage(jsonMsg);
                    } catch (const std::exception &e) {
                        toastError(std::format("Failed to parse socket message: {}", e.what()));
                    }
                    buffer.clear();
                }
            } else {
                buffer.push_back(readBuf[i]);
            }
        }
    }
}

void sendCommand(const int sock, const std::string &message) {
    size_t totalSent = 0;
    while (totalSent < message.size()) {
        // toast(std::format("Sending data... ({} / {})", totalSent, message.size()));
        const ssize_t sent = send(sock, message.c_str() + totalSent, message.size() - totalSent, 0);
        if (sent == -1) {
            toastError(std::format("Failed to send message: {}", strerror(errno)));
            std::this_thread::sleep_for(std::chrono::seconds(1));
            continue;
        }
        // toast(std::format("Sent {} bytes", sent));
        totalSent += sent;
    }
    constexpr char nullByte = '\0';
    send(sock, &nullByte, 1, 0);
}

std::atomic g_shouldStop{false};
std::thread g_listenerThread;

void startSocketListener() {
    // Join previous thread if it exists and is joinable
    if (g_listenerThread.joinable()) {
        g_shouldStop = true;
        g_listenerThread.join();
    }

    g_shouldStop = false;
    g_listenerThread = std::thread([] {
        auto socketPath = getSocketPath();

        while (!g_shouldStop.load()) {
            const int sock = socket(AF_UNIX, SOCK_STREAM, 0);
            if (sock == -1) {
                toastError(std::format("Failed to create socket: {}", strerror(errno)));
                std::this_thread::sleep_for(std::chrono::seconds(1));
                continue;
            }

            sockaddr_un addr = {};
            addr.sun_family = AF_UNIX;
            strncpy(addr.sun_path, socketPath.c_str(), sizeof(addr.sun_path) - 1);

            if (connect(sock, reinterpret_cast<sockaddr *>(&addr), sizeof(addr)) == -1) {
                close(sock);
                toastError(std::format("Failed to connect to socket ({}): {}", socketPath, strerror(errno)));
                std::this_thread::sleep_for(std::chrono::seconds(1));
                continue;
            }
            toast(std::format("Connected to socket: {}", socketPath));

            std::string message = "\"GetConfigWatch\"";
            sendCommand(sock, message);

            listenOnSocket(g_shouldStop, sock);
            close(sock);
            if (!g_shouldStop.load()) {
                std::this_thread::sleep_for(std::chrono::seconds(1));
            }
        }
    });
}

void stopSocketListener() {
    g_shouldStop = true;
    if (g_listenerThread.joinable()) {
        g_listenerThread.join();
    }
    toast("Socket listener stopped");
}
