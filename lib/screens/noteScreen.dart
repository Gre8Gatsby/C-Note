import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'dart:convert';

class NoteScreen extends StatefulWidget {
  const NoteScreen({Key? key}) : super(key: key);

  @override
  _NoteScreenState createState() => _NoteScreenState();
}

class _NoteScreenState extends State<NoteScreen> {
  List<CallNote> callNotes = [];
  DateTime selectedDate = DateTime.now();

  @override
  void initState() {
    super.initState();
    _loadCallNotes();
  }

  Future<void> _loadCallNotes() async {
    final prefs = await SharedPreferences.getInstance();
    final allKeys = prefs.getKeys();
    List<CallNote> notes = [];

    for (String key in allKeys) {
      if (key.contains('_')) {
        final value = prefs.getString(key);
        if (value != null) {
          final Map<String, dynamic> noteMap = json.decode(value);
          notes.add(CallNote(
            contactName: key.split('_')[0],
            phoneNumber: noteMap['phoneNumber'],
            timestamp: DateTime.parse(noteMap['timestamp']),
            summary: noteMap['summary'],
          ));
        }
      }
    }

    setState(() {
      callNotes = notes..sort((a, b) => b.timestamp.compareTo(a.timestamp));
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: Text('Call Notes', style: TextStyle(color: Colors.black)),
        backgroundColor: Colors.white,
        elevation: 0,
        actions: [
          Padding(
            padding: const EdgeInsets.all(8.0),
            child: Image.asset(
              'images/verbalyze.png',
              width: 40,
              height: 40,
            ),
          ),
        ],
      ),
      body: Column(
        children: [
          _buildCalendar(),
          Expanded(
            child: _buildNotesList(),
          ),
        ],
      ),
    );
  }

  Widget _buildCalendar() {
    return Container(
      padding: EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            DateFormat('MMMM, d').format(selectedDate),
            style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold, color: Colors.black),
          ),
          SizedBox(height: 10),
          SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: Row(
              children: List.generate(7, (index) {
                final date = DateTime.now().subtract(Duration(days: 3 - index));
                return GestureDetector(
                  onTap: () => setState(() => selectedDate = date),
                  child: Container(
                    width: 40,
                    height: 60,
                    margin: EdgeInsets.symmetric(horizontal: 5),
                    decoration: BoxDecoration(
                      color: date.day == selectedDate.day ? Colors.blue.shade100 : Colors.transparent,
                      borderRadius: BorderRadius.circular(10),
                    ),
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Text(DateFormat('E').format(date)[0], style: TextStyle(fontSize: 12, color: Colors.black)),
                        Text('${date.day}', style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold, color: Colors.black)),
                      ],
                    ),
                  ),
                );
              }),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildNotesList() {
    final filteredNotes = callNotes.where((note) => 
      note.timestamp.year == selectedDate.year &&
      note.timestamp.month == selectedDate.month &&
      note.timestamp.day == selectedDate.day
    ).toList();

    return ListView.builder(
      itemCount: filteredNotes.length,
      itemBuilder: (context, index) {
        final note = filteredNotes[index];
        return Container(
          margin: EdgeInsets.symmetric(vertical: 10, horizontal: 20),
          padding: EdgeInsets.all(15),
          decoration: BoxDecoration(
            color: Colors.blue.shade100,
            borderRadius: BorderRadius.circular(15),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text(
                    note.contactName,
                    style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold, color: Colors.black),
                  ),
                  Text(
                    DateFormat('HH:mm').format(note.timestamp),
                    style: TextStyle(fontSize: 14, color: Colors.grey),
                  ),
                ],
              ),
              SizedBox(height: 5),
              Text(
                note.phoneNumber,
                style: TextStyle(fontSize: 14, color: Colors.grey),
              ),
              SizedBox(height: 10),
              Text(
                note.summary,
                style: TextStyle(fontSize: 14, color: Colors.black),
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
              SizedBox(height: 10),
              Text(
                'See More',
                style: TextStyle(fontSize: 14, color: Colors.blue),
              ),
            ],
          ),
        );
      },
    );
  }
}

class CallNote {
  final String contactName;
  final String phoneNumber;
  final DateTime timestamp;
  final String summary;

  CallNote({
    required this.contactName,
    required this.phoneNumber,
    required this.timestamp,
    required this.summary,
  });
}
