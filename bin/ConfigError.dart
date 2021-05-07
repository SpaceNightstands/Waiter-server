import 'package:Waiter/Error.dart' as waiter_error;

class ConfigError extends waiter_error.Error {
  const ConfigError(String message) : super('Configuration', message);
}
