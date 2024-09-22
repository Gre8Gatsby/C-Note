import 'package:flutter/material.dart';
import 'package:c_note/utils.dart';
import 'package:c_note/gpt_service.dart';

class SummaryScreen extends StatefulWidget {
  const SummaryScreen({super.key});

  @override
  _SummaryScreenState createState() => _SummaryScreenState();
}

class _SummaryScreenState extends State<SummaryScreen> {
  final TextEditingController _textController = TextEditingController();
  String _summary = '';
  bool _isLoading = false;

  Future<void> _generateSummary() async {
    if (_textController.text.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Please enter some text to summarize')),
      );
      return;
    }

    setState(() {
      _isLoading = true;
    });

    try {
      final summary = await GPTService.generateSummary(_textController.text);
      setState(() {
        _summary = summary;
        _isLoading = false;
      });
    } catch (e) {
      setState(() {
        _isLoading = false;
      });
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Failed to generate summary: $e')),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Summary', style: Utils.getCustomFont(fontFamily: 'Roboto', fontSize: 20, fontWeight: FontWeight.bold, color: Colors.white)),
        flexibleSpace: Container(
          decoration: const BoxDecoration(
            gradient: LinearGradient(
              begin: Alignment(-1, -1),
              end: Alignment(1, 1),
              colors: <Color>[Color(0xff23a5d6), Color(0xff623aa2)],
              stops: <double>[0, 1],
            ),
          ),
        ),
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            TextField(
              controller: _textController,
              maxLines: 5,
              decoration: const InputDecoration(
                hintText: 'Enter text to summarize',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: _isLoading ? null : _generateSummary,
              child: _isLoading
                  ? const CircularProgressIndicator(color: Colors.white)
                  : const Text('Generate Summary'),
            ),
            const SizedBox(height: 16),
            Text(
              'Summary:',
              style: Utils.getCustomFont(fontFamily: 'Roboto', fontSize: 18, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 8),
            Expanded(
              child: SingleChildScrollView(
                child: Text(
                  _summary,
                  style: Utils.getCustomFont(fontFamily: 'Roboto', fontSize: 16),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
