import 'dart:io';
import 'package:flutter/material.dart';
import 'package:verbalyze/screens/textFileScreen.dart';

class summaryScreen extends StatelessWidget {
  Future<List<FileSystemEntity>> fetchTextFiles() async {
    Directory directory = Directory('/storage/emulated/0/Android/data/com.example.untitled/files');
    List<FileSystemEntity> fileList = directory.listSync().where((file) => file is File && file.path.endsWith('.txt')).toList();
    return fileList;
  }

  @override
  Widget build(BuildContext context) {
    double baseWidth = 360;
    double fem = MediaQuery.of(context).size.width / baseWidth;
    return Scaffold(
      appBar: AppBar(title: Text('Text File List'),

    flexibleSpace: Container(
    decoration: BoxDecoration(
      gradient:  LinearGradient (
        begin:  Alignment(-1, -1),
        end:  Alignment(1, 1),
        colors:  <Color>[Color(0xff23a5d6), Color(0xff623aa2)],
        stops:  <double>[0, 1],
      ),),),),
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
                crossAxisCount: 3, // Number of columns in the grid
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
                    margin: EdgeInsets.fromLTRB(8.6*fem, 8.6*fem, 8.6*fem, 13.66*fem), // Add margin
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(12 * fem), // Add border radius
                    ),
                    child: Container(
                      decoration: BoxDecoration (
                    borderRadius:  BorderRadius.circular(12*fem),
                    gradient:  LinearGradient (
                      begin:  Alignment(-1, -1),
                      end:  Alignment(1, 1),
                      colors:  <Color>[Color(0xff23a5d6), Color(0xff623aa2)],
                      stops:  <double>[0, 1],
                    ),
                  ),
                      alignment: Alignment.center, // Center the content
                      padding: EdgeInsets.symmetric(vertical: 20, horizontal: 34,),
                      child: Text(
                        fileName,
                        style: TextStyle(
                          fontSize: 10,
                          fontWeight: FontWeight.bold,
                          color: Colors.white, // White text color
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

