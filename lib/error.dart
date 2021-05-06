class Error {
  final String reason;
  final String message;

  Error(this.reason, this.message);

  Map<String, String> toJson() => {
        'reason': reason,
        'message': message,
      };
}
