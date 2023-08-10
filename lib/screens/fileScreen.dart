import 'package:flutter/material.dart';
import 'dart:io';
import 'package:path_provider/path_provider.dart';

class fileScreen extends StatefulWidget {
  const fileScreen({Key? key}) : super(key: key);

  @override
  State<fileScreen> createState() => _fileScreenState();
}

class _fileScreenState extends State<fileScreen>{
  String directory="";
  List file=[];

  get io => null;

  @override
  void initState(){
    super.initState();
    _listOfFile();
  }

  void _listOfFile() async{
    directory="/storage/emulated/0/Android/data/com.example.untitled";
    setState(() {
      file=io.Directory('$directory/files').listSync();
    });
  }

  @override
  Widget build(BuildContext context){
    return MaterialApp(
      title: 'List of Files',
      home: Scaffold(
        appBar: AppBar(
          title: Text("Get call summary"),
        ),
        body: Container(
            child: Column(
                children:<Widget>[
                  Expanded(child: ListView.builder(
                    itemCount: file.length,
                    itemBuilder: (BuildContext context, int index){
                      return Text(file[index].toString());
                    },
                  ))

                ])
        ),
      ),
    );
  }

}
