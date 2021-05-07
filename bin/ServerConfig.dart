import 'SocketAddress.dart';

class ServerConfig {
  final SocketAddress socket;
  final String corsOrigin;
  final String jwtKey;

  const ServerConfig(this.socket, {this.corsOrigin = '*', required this.jwtKey});
}
