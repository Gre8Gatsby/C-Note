import 'package:flutter/material.dart';
import 'package:speech_to_text/speech_to_text.dart';
import 'dart:io';
import 'package:path_provider/path_provider.dart';
import 'package:intl/intl.dart';

class noteScreen extends StatefulWidget {
  const noteScreen({Key? key}) : super(key: key);

  @override
  State<noteScreen> createState() => _noteScreenState();
}

class _noteScreenState extends State<noteScreen> {
  SpeechToText speechToText= SpeechToText();
  String text = "Hold the button to take note";
  var isListening=false;

  Future<void> _saveTextToFile() async{
    try{
      Directory? dir = await getExternalStorageDirectory();
      print(dir?.path);


      String timestamp = DateFormat('yyyyMMdd_HHmmss').format(DateTime.now());

      File file = File('${dir?.path}/verbalyze$timestamp.txt'); // Use the timestamp in the filename
      await file.writeAsString(text);

      showDialog(
        context: context,
        builder: (context) => AlertDialog(
          title: Text('File saved'),
          actions: [
            ElevatedButton(
              onPressed: () => Navigator.pop(context),
              child: Text('OK'),
            ),
          ],
        ),
      );

    }
    catch(e){
      showDialog(context: context, builder: (context) => AlertDialog(
        title: Text('Error'),
        actions: [
          ElevatedButton(onPressed: ()=> Navigator.pop(context), child: Text('OK'))
        ],
      ));
    }
  }
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      floatingActionButtonLocation: FloatingActionButtonLocation.centerFloat,
      floatingActionButton: Column(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          FloatingActionButton(
            onPressed: () async {
              if (!isListening) {
                var available = await speechToText.initialize();
                if (available) {
                  setState(() {
                    isListening = true;
                    speechToText.listen(
                      onResult: (result) {
                        setState(() {
                          text = result.recognizedWords;
                        });
                      },
                    );
                  });
                }
              }
            },
            backgroundColor: Colors.blue,
            child: Icon(Icons.mic),
          ),
          SizedBox(height: 16),
          FloatingActionButton(
            onPressed: () {
              setState(() {
                isListening = false;
              });
              speechToText.stop();
              _saveTextToFile();
            },
            backgroundColor: Colors.red,
            child: Icon(Icons.stop),
          ),
        ],
      ),
      appBar: AppBar(
        leading: Icon(Icons.sort_outlined, color: Colors.white),
        centerTitle: true,
        backgroundColor: Colors.blue,
        title: Text('Verbalyze'),
      ),
      body: SingleChildScrollView(
        reverse: true,
        physics: BouncingScrollPhysics(),
        child: Container(
          width: MediaQuery.of(context).size.width,
          height: MediaQuery.of(context).size.height * 0.7,
          alignment: Alignment.center,
          color: Colors.white70,
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
          margin: const EdgeInsets.only(bottom: 150),
          child: Text(
            text,
            style: TextStyle(
              fontSize: 24,
              color: isListening ? Colors.black87 : Colors.black54,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
      ),
    );

  }
}

