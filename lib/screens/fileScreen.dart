import 'dart:io';
import 'package:flutter/material.dart';

class fileScreen extends StatelessWidget {
  Future<List<FileSystemEntity>> fetchTextFiles() async {
    Directory directory = Directory('/storage/emulated/0/Android/data/com.example.untitled/files');
    List<FileSystemEntity> fileList = directory.listSync().where((file) => file is File && file.path.endsWith('.txt')).toList();
    return fileList;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('Text File List')),
      body: FutureBuilder<List<FileSystemEntity>>(
        future: fetchTextFiles(),
        builder: (context, snapshot) {
          if (snapshot.connectionState == ConnectionState.waiting) {
            return Center(child: CircularProgressIndicator());
          } else if (snapshot.hasError) {
            return Center(child: Text('Error: ${snapshot.error}'));
          } else if (!snapshot.hasData || snapshot.data!.isEmpty) {
            return Center(child: Text('No text files found.'));
          } else {
            return GridView.builder(
              gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                crossAxisCount: 2, // Number of columns in the grid
              ),
              itemCount: snapshot.data!.length,
              itemBuilder: (context, index) {
                FileSystemEntity file = snapshot.data![index];
                String fileName = file.uri.pathSegments.last; // Get the last segment of the file path
                return GestureDetector(
                  onTap: () {
                    _openTextFile(context, file);
                  },
                  child: Card(
                    color: Colors.blue, // Blue background color
                    child: Center(
                      child: Text(
                        fileName,
                        style: TextStyle(
                          color: Colors.white, // White text color
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ),
                  ),
                );
              },
            );
          }
        },
      ),
    );
  }

  void _openTextFile(BuildContext context, FileSystemEntity file) async {
    if (await file.exists()) {
      String content = await File(file.path).readAsString();
      Navigator.push(
        context,
        MaterialPageRoute(
          builder: (context) => TextFileContentScreen(content),
        ),
      );
    }
  }
}

class TextFileContentScreen extends StatelessWidget {
  final String content;

  TextFileContentScreen(this.content);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('Text File Content')),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: SingleChildScrollView(
          child: Text(content),
        ),
      ),
    );
  }
}
