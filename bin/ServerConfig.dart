import 'SocketAddress.dart';

class ServerConfig {
  final SocketAddress socket;
  final String corsOrigin;
  final String jwtKey;
  final List<String> allowedSubjects;

  const ServerConfig(
    this.socket,
    {
      this.corsOrigin = '*',
      required this.jwtKey,
      this.allowedSubjects = const []
    }
  );
}
