class GPTService {
  static Future<String> summarizeCall(String transcript) async {
    // TODO: Implement actual GPT API call for summarization
    // This is a placeholder implementation
    await Future.delayed(Duration(seconds: 1)); // Simulate API call
    return "Summarized content of: $transcript";
  }
}