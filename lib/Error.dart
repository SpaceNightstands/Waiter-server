class Error {
  final String reason;
  final String message;

  const Error(this.reason, this.message);

  Map<String, String> toJson() => {
        'reason': reason,
        'message': message,
      };
}
