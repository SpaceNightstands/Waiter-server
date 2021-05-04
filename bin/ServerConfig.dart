import 'SocketAddress.dart';

class ServerConfig {
  final SocketAddress socket;
  final String corsOrigin;

  const ServerConfig(this.socket, {this.corsOrigin = '*'});
}
