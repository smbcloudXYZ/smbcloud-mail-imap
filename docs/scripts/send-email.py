import smtplib
from email import encoders
from email.mime.base import MIMEBase
from email.mime.multipart import MIMEMultipart
from email.mime.text import MIMEText

smtp_server = "localhost"
smtp_port = 2525
from_email = "you@example.com"
to_email = "to@example.com"
subject = "Test Email"
body = "This is a test email sent to my local SMTP server on port 2525."
html_body = "<html><body>This is a HTML</body></html>"

# Create the email
message = MIMEMultipart()
message["From"] = from_email
message["To"] = to_email
message["Subject"] = subject

message.attach(MIMEText(body, "plain"))
message.attach(MIMEText(html_body, "html"))

# to add an attachment:
file_path = "myfile.txt"
with open(file_path, "rb") as attachment:
    part = MIMEBase("application", "octet-stream")
    part.set_payload(attachment.read())
    encoders.encode_base64(part)
    part.add_header(
        "Content-Disposition",
        f"attachment; filename= {file_path}",
    )
    message.attach(part)

# Send the email
try:
    with smtplib.SMTP(smtp_server, smtp_port) as server:
        _ = server.starttls()
        _ = server.sendmail(from_email, to_email, message.as_string())
    print("Email sent successfully")
except Exception as e:
    print(f"Failed to send email: {e}")
