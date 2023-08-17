import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:verbalyze/screens/splashScreen.dart';
import '../utils.dart';
import 'package:fluttertoast/fluttertoast.dart';

class LoginPage extends StatefulWidget {
  @override
  _LoginPageState createState() => _LoginPageState();
}

class _LoginPageState extends State<LoginPage> {
  TextEditingController emailController = TextEditingController();
  TextEditingController passwordController = TextEditingController();

  void _login() async {
    SharedPreferences prefs = await SharedPreferences.getInstance();
    String storedEmail = prefs.getString('email') ?? '';
    String storedPassword = prefs.getString('password') ?? '';
    prefs.setBool(SplashScreenState.KEYLOGIN,true);

    if (emailController.text == storedEmail && passwordController.text == storedPassword) {
      Navigator.pushReplacementNamed(context, '/home'); // Navigate to home screen
    } else {
      // Show an error message or toast indicating failed authentication
      // For simplicity, we'll just print a message here.
      Fluttertoast.showToast(
        msg: 'Invalid credentials',
        toastLength: Toast.LENGTH_SHORT,
        gravity: ToastGravity.CENTER,
      );
      print('Authentication failed');
    }
  }

  @override
  Widget build(BuildContext context) {
    double baseWidth = 360;
    double fem = MediaQuery.of(context).size.width / baseWidth;
    double ffem = fem * 0.97;
    return Scaffold(
      body: SingleChildScrollView(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              // vector1gWV (1:66)
              margin: EdgeInsets.fromLTRB(0 * fem, 0 * fem, 0 * fem, 13 * fem),
              width: 391 * fem,
              height: 119 * fem,
              child: Image.asset(
                'images/vector-1-Nm7.png',
                width: 391 * fem,
                height: 119 * fem,
              ),
            ),
            SizedBox(height: 90),
            Text(
              // createaccountaLy (1:77)
              'Login',
              textAlign: TextAlign.center,
              style: SafeGoogleFont(
                'Lato',
                fontSize: 30 * ffem,
                fontWeight: FontWeight.w700,
                height: 1.2 * ffem / fem,
                color: Color(0xff262626),
              ),
            ),
            SizedBox(height: 35),
            Padding(
              padding: EdgeInsets.symmetric(horizontal: 20.0),
              child: TextField(
                  controller: emailController,
                  decoration: InputDecoration(
                    labelText: 'Email',
                    hintText: 'Mail Address',
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(
                          25.0),
                    ),
                    prefixIcon: Icon(Icons.alternate_email),
                  )),
            ),
            SizedBox(height: 20),
            Padding(
              padding: EdgeInsets.symmetric(horizontal: 20.0),
              child: TextField(
                  controller: passwordController,
                  decoration: InputDecoration(
                    labelText: 'Password',
                    hintText: 'Your Password',
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(
                          25.0),
                    ),
                    prefixIcon: Icon(Icons.password),
                  )),
            ),
            SizedBox(height: 20),
            ElevatedButton(onPressed: _login, child: Text('Login')),
            SizedBox(height: 10),
            TextButton(
              onPressed: () {
                Navigator.pushReplacementNamed(context, '/register'); // Navigate to register page
              },
              child: Text("Don't have an account? Register"),
            ),
            
          ],
        ),
      ),
    );
  }
}
