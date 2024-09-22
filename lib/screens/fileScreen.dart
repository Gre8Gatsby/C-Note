import 'dart:io';
import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';
import 'package:c_note/utils.dart';

class FileScreen extends StatefulWidget {
  const FileScreen({super.key});

  @override
  State<FileScreen> createState() => _FileScreenState();
}

class _FileScreenState extends State<FileScreen> {
  List<FileSystemEntity> _files = [];

  @override
  void initState() {
    super.initState();
    _loadFiles();
  }

  Future<void> _loadFiles() async {
    final directory = await getExternalStorageDirectory();
    if (directory != null) {
      final files = directory.listSync()
          .where((entity) => entity is File && entity.path.endsWith('.txt'))
          .toList();
      setState(() {
        _files = files;
      });
    }
  }

  Future<void> _viewFile(File file) async {
    final content = await file.readAsString();
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(file.path.split('/').last),
        content: SingleChildScrollView(
          child: Text(content),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Saved Notes'),
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
      body: _files.isEmpty
          ? Center(
              child: Text(
                'No saved notes found.',
                style: Utils.getCustomFont(
                  fontFamily: 'Roboto',
                  fontSize: 18,
                  color: Colors.grey,
                ),
              ),
            )
          : ListView.builder(
              itemCount: _files.length,
              itemBuilder: (context, index) {
                final file = _files[index] as File;
                return ListTile(
                  title: Text(
                    file.path.split('/').last,
                    style: Utils.getCustomFont(
                      fontFamily: 'Roboto',
                      fontSize: 16,
                    ),
                  ),
                  onTap: () => _viewFile(file),
                );
              },
            ),
    );
  }
}
