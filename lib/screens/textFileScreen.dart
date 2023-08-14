import 'package:flutter/material.dart';

class TextFileContentScreen extends StatelessWidget {
  final String content;

  TextFileContentScreen(this.content);

  @override
  Widget build(BuildContext context) {
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
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: SingleChildScrollView(
          child: Text(content),
        ),
      ),
    );
  }
}
