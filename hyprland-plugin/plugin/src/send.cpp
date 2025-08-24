#include "send.h"

#include <sys/socket.h>
#include <sys/un.h>
#include <cstring>

#include "defs.h"

void sendStringToHyprshellSocket(const std::string &message) {
    const int sockfd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (sockfd < 0) return;
    sockaddr_un addr{};
    addr.sun_family = AF_UNIX;
    std::strncpy(addr.sun_path, HYPRSHELL_SOCKET_PATH, sizeof(addr.sun_path) - 1);
    if (connect(sockfd, reinterpret_cast<sockaddr *>(&addr), sizeof(addr)) < 0) {
        close(sockfd);
        return;
    }
    send(sockfd, message.c_str(), message.size(), 0);
    close(sockfd);
}
