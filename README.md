# Verbalyze - Automated Call Note-Taking App

Verbalyze is a Flutter-based mobile application designed to automatically record, transcribe, and summarize phone calls. It uses advanced speech recognition and natural language processing to provide users with concise, actionable notes from their conversations.

## Features

- **Automatic Call Recording**: Captures audio from phone calls in real-time.
- **Speech-to-Text Transcription**: Converts spoken words into written text.
- **AI-Powered Summarization**: Uses GPT to generate concise summaries of call content.
- **Contact Integration**: Associates call notes with contacts from the user's phone book.
- **Calendar View**: Organizes call notes in an easy-to-navigate calendar interface.
- **Background Service**: Runs in the background to capture calls without active user intervention.
- **Data Privacy**: Ensures user data is securely stored and processed.

## Getting Started

### Prerequisites

- Flutter SDK (version 2.0 or later)
- Dart (version 2.12 or later)
- Android Studio or VS Code with Flutter extensions
- An Android device or emulator (iOS support coming soon)

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/verbalyze.git
   ```

2. Navigate to the project directory:
   ```
   cd verbalyze
   ```

3. Install dependencies:
   ```
   flutter pub get
   ```

4. Run the app:
   ```
   flutter run
   ```

## Project Structure

- `lib/`: Contains the main Dart code for the application
  - `screens/`: Individual screen widgets (e.g., NoteScreen, SummaryScreen)
  - `services/`: Background services and API integrations
  - `utils/`: Utility functions and helper classes
- `android/`: Android-specific configurations
- `ios/`: iOS-specific configurations (for future use)
- `assets/`: Static assets like images and fonts

## Configuration

To use the GPT API for summarization, you need to set up your API credentials:

1. Create a file named `lib/google_cloud_config.dart`
2. Add your Google Cloud credentials to this file (see `google_cloud_config_example.dart` for format)

## Contributing

We welcome contributions to Verbalyze! Please read our [Contributing Guide](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Flutter team for the excellent cross-platform framework
- OpenAI for the GPT API used in summarization
- All contributors and supporters of the project

## Contact

For any queries or suggestions, please open an issue on this repository or contact the maintainers directly.

---

Happy note-taking with Verbalyze!
