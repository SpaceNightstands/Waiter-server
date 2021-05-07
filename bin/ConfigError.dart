import '../lib/error.dart' as waiter_error;

class ConfigError extends waiter_error.Error {
  ConfigError(String message) : super('Configuration', message);
}
